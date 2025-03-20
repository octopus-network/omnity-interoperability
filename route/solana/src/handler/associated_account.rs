use std::str::FromStr;

use ic_spl::token::associated_account::get_associated_token_address_with_program_id;
use ic_spl::token::constants::token_program_id;

use ic_solana::types::Pubkey;

use crate::solana_client::solana_rpc::TxError;
use crate::solana_client::create_ata;
use crate::solana_client::solana_client;
use crate::call_error::Reason;
use crate::state::TxStatus;
use crate::state::{mutate_state, read_state};
use crate::state::{AccountInfo, AtaKey};

use ic_solana::types::TransactionConfirmationStatus;

use crate::constants::{RETRY_4_BUILDING, RETRY_4_STATUS, TAKE_SIZE};
use ic_canister_log::log;
use ic_solana::logs::{DEBUG, ERROR, WARNING};

pub async fn create_associated_account() {
    let tickets = read_state(|s| {
        s.tickets_queue
            .iter()
            .take(TAKE_SIZE.try_into().unwrap())
            .map(|(seq, ticket)| (seq, ticket))
            .collect::<Vec<_>>()
    });
    let mut creating_atas = vec![];
    read_state(|s| {
        for (_seq, ticket) in tickets.into_iter() {
            if let Some(token_mint) = s.token_mint_accounts.get(&ticket.token) {
                //the token mint account must be Finalized
	            if matches!(token_mint.status, TxStatus::Finalized) {
		            match s.associated_accounts.get(&AtaKey {
			            owner: ticket.receiver.to_string(),
			            token_mint: token_mint.account.to_string(),
		            }) {
			            None => {
				            creating_atas.push((ticket.receiver.to_owned(), token_mint.to_owned()))
			            }
			            Some(ata) => {
				            //filter account,unconformed and retry < RETRY_LIMIT_SIZE
				            if !matches!(ata.status, TxStatus::Finalized)
					            && ata.retry_4_building < RETRY_4_BUILDING
				            {
					            creating_atas
						            .push((ticket.receiver.to_owned(), token_mint.to_owned()))
				            }
			            }
		            }
                }
            }
        }
    });

    // let sol_client = solana_client().await;
    for (owner, token_mint) in creating_atas.into_iter() {
        let to_account_pk = Pubkey::from_str(owner.as_str()).expect("Invalid to_account address");
        let token_mint_pk =
            Pubkey::from_str(token_mint.account.as_str()).expect("Invalid token_mint address");

        let associated_account = if let Some(account) = read_state(|s| {
	        s.associated_accounts.get(&AtaKey {
		        owner: owner.to_string(),
		        token_mint: token_mint.account.to_string(),
	        })
        }) {
            account
        } else {
            let associated_account = get_associated_token_address_with_program_id(
                &to_account_pk,
                &token_mint_pk,
                &token_program_id(),
            );
            log!(
                DEBUG,  
                "[associated_account::create_associated_account] native associated_account: {:?} for {:?} and {:?}",
                associated_account,owner,token_mint.account
            );
            let new_account_info = AccountInfo {
                account: associated_account.to_string(),
                retry_4_building: 0,
	            retry_4_status: 0,
                signature: None,
                status: TxStatus::New,
            };
            //save inited account info
            mutate_state(|s| {
                s.associated_accounts.insert(
	                AtaKey {
		                owner: owner.to_string(),
		                token_mint: token_mint.account.to_string(),
	                },
                    new_account_info.to_owned(),
                )
            });
            // associated_account
            new_account_info
        };

        log!(
            DEBUG,
            "[associated_account::create_associated_account] ata_account_info from solana route : {:?} ",
            associated_account);
	    
	    // check ATA exists on solana
	    let sol_client = solana_client().await;
	    let ata_account_info = sol_client
            .get_account_info(associated_account.account.to_string())
            .await;
	    log!(
             DEBUG,
             "[associated_account::create_associated_account] ata_account_info: {:?} account info from solana: {:?} ",
             associated_account,ata_account_info,
         );
	    if let Ok(account_info) = ata_account_info {
		    if matches!(account_info, Some(..)) {
			    let ata = AccountInfo {
				    account: associated_account.account.to_string(),
				    retry_4_building: associated_account.retry_4_building,
				    retry_4_status: associated_account.retry_4_status,
				    signature: associated_account.signature,
				    status: TxStatus::Finalized,
			    };
			    //update ata info
			    mutate_state(|s| {
                    s.associated_accounts.insert(
	                    AtaKey {
		                    owner: owner.to_string(),
		                    token_mint: token_mint.account.to_string(),
	                    },
                        ata,
                    )
                });
			    //skip this ata
			    continue;
		    }
	    }

        match &associated_account.status {
            TxStatus::New => {
                match &associated_account.signature {
	                // not exists,create it
                    None => {
	                    build_ata(owner.to_owned(), token_mint.account.to_string()).await;
                    }
                    Some(sig) => {
                        log!(
                            DEBUG,
                            "[associated_account::create_associated_account] The ata ({:?}) already submited and waiting for the tx({:?}) to be finallized! ",
                            associated_account.account.to_string(),
                            sig
                        );
                        // update ata status
	                    update_ata_status(
		                    sig.to_string(),
		                    owner.to_string(),
		                    token_mint.account.to_string(),
	                    )
		                    .await;
                    }
                }
            }
            TxStatus::Pending => {
                match &associated_account.signature {
                    // nothing to do ,just waiting
	                None => {
		                //   handle_creating_ata(owner.to_owned(), token_mint.account.to_string()).await;
		                log!(
                        DEBUG,
                        "[associated_account::create_associated_account] the associated account ({:?}) is creating,pls wait ...",
                        associated_account
                    );
	                }
	                Some(sig) => {
		                log!(
                           DEBUG,
                           "[associated_account::create_associated_account] the ata {:?} already submited and waiting for the tx({:?}) to be finallized! ",
                           associated_account.account.to_string(),
                           sig
                       );
		                // update ata status
		                update_ata_status(
			                sig.to_string(),
			                owner.to_string(),
			                token_mint.account.to_string(),
		                )
			                .await;
	                }
                }
            }
            TxStatus::Finalized => {
                log!(
                    DEBUG,
                    "[associated_account::create_associated_account] {:?} already finalized !",
                    associated_account,
                );
            }
            TxStatus::TxFailed { e } => {
                log!(
                    DEBUG,
                   "[associated_account::create_associated_account] failed to create_associated_account for owner: {} and token mint: {}, error: {:}",
                   owner,token_mint.account,e
                );
                // handle_creating_ata(owner.to_string(), token_mint.account.to_string()).await;
                match &associated_account.signature {
                    // not exists,create it
	                None => {
		                build_ata(owner.to_owned(), token_mint.account.to_string()).await;
	                }
	                Some(sig) => {
		                log!(
                           DEBUG,
                           "[associated_account::create_associated_account] the ata {:?} already submited and waiting for the tx({:?}) to be finallized! ",
                           associated_account.account.to_string(),
                           sig
                       );
		                // update ata status
		                update_ata_status(
			                sig.to_string(),
			                owner.to_string(),
			                token_mint.account.to_string(),
		                )
			                .await;
	                }
                }
            }
        }
    }
}

pub async fn build_ata(owner: String, mint_address: String) {
    match create_ata(owner.to_string(), mint_address.to_string()).await {
        Ok(sig) => {
            log!(
                DEBUG,
                "[associated_account::build_ata] create_ata signature : {:?}",
                sig
            );
            // update account created signature and retry ,but not confirmed
            mutate_state(|s| {
	            let ata_key = AtaKey {
		            owner: owner.to_string(),
		            token_mint: mint_address.to_string(),
	            };
	            if let Some(account) = s.associated_accounts.get(&ata_key).as_mut() {
                    account.signature = Some(sig.to_string());
                    // account.retry_4_building += 1;
                    s.associated_accounts.insert(ata_key, account.to_owned());
                }
            });
	        // update ata status
            //  update_ata_status(sig.to_string(),owner.to_string(),mint_address.to_string()).await;
        }
        Err(e) => {
            log!(
                ERROR,
                "[associated_account::build_ata] create_ata for owner: {:} and token_mint: {:}, error: {:?}  ",
                owner.to_string(), mint_address.to_string(), e
            );
	        let tx_error = match e.reason {
		        Reason::QueueIsFull
		        | Reason::OutOfCycles
		        | Reason::CanisterError(_)
		        | Reason::Rejected(_) => todo!(),
                Reason::TxError(tx_error) => tx_error,
            };
	        // update account retry
            mutate_state(|s| {
	            let ata_key = AtaKey {
		            owner: owner.to_string(),
		            token_mint: mint_address.to_string(),
	            };
	            if let Some(account) = s.associated_accounts.get(&ata_key).as_mut() {
		            account.status = TxStatus::TxFailed { e: tx_error };
		            account.retry_4_building += 1;
		            //reset signature
		            account.signature = None;
		            s.associated_accounts.insert(ata_key, account.to_owned());
                }
            });
        }
    }
}

pub async fn update_ata_status(sig: String, owner: String, mint_address: String) {
	let tx_status_ret = crate::solana_client::get_signature_status(vec![sig.to_string()]).await;
	match tx_status_ret {
		Err(e) => {
			log!(
                WARNING,
                "[associated_account::update_ata_status] get_signature_status for {} ,err: {:?}",
                sig.to_string(),
                e
            );
			let tx_error = match e.reason {
				Reason::QueueIsFull
				| Reason::OutOfCycles
				| Reason::TxError(_)
				| Reason::Rejected(_) => todo!(),
				Reason::CanisterError(tx_error) => tx_error,
			};
			
			mutate_state(|s| {
				let ata_key = AtaKey {
					owner: owner.to_string(),
					token_mint: mint_address.to_string(),
				};
				if let Some(account) = s.associated_accounts.get(&ata_key).as_mut() {
					// if update statue is up to the RETRY_4_STATUS and the tx is droped, rebuild the account
					if account.retry_4_status > RETRY_4_STATUS {
						log!(
                    WARNING,
                    "[token_account::update_ata_status] retry for get_signature_status up to limit size :{} ,and need to rebuild the account",
                    RETRY_4_STATUS,);
						account.status = TxStatus::New;
						account.retry_4_building = 0;
						account.retry_4_status = 0;
						account.signature = None;
						s.associated_accounts.insert(ata_key, account.to_owned());
					} else {
						account.retry_4_status += 1;
						account.status = TxStatus::TxFailed {
							e: TxError {
								block_hash: String::default(),
								signature: sig.to_owned(),
								error: tx_error.to_owned(),
							},
						};
						s.associated_accounts.insert(ata_key, account.to_owned());
					}
				}
			});
		}
		Ok(status_vec) => {
			status_vec.first().map(|tx_status| {
				log!(
                    DEBUG,
                    "[associated_account::update_ata_status] signature {}  status : {:?} ",
                    sig.to_string(),
                    tx_status,
                );
                if let Some(status) = &tx_status {
                    if matches!(
                        status.confirmation_status,
                        Some(TransactionConfirmationStatus::Finalized)
                    ) {
						// update account status to Finalized
						mutate_state(|s| {
							let ata_key = AtaKey {
								owner: owner.to_string(),
								token_mint: mint_address.to_string(),
							};
							if let Some(account) = s.associated_accounts.get(&ata_key).as_mut() {
                                account.status = TxStatus::Finalized;
                                s.associated_accounts.insert(ata_key, account.to_owned());
                            }
						});
					}
				}
			});
		}
	}
}
