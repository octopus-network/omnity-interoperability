#!/usr/bin/env python


'''
This script is mainly used to send a tx to solana, 
the tx operation includes transfer, burn and memo three instructions.
Before executing this script, please install the related dependency libraries:
pip install solders
pip install solana
'''

import argparse
from solana.rpc.api import Client
from solders.pubkey import Pubkey
from solders.keypair import Keypair
import solders.system_program as sp
from solders.message import MessageV0, Message
from solders.transaction import Transaction
from solana.rpc.types import  TxOpts
from spl.token.client import Token
from spl.token.constants import ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID
import spl.token.instructions as spl_token
from spl.memo.instructions import MemoParams, create_memo
from spl.memo.constants import MEMO_PROGRAM_ID
from solana.rpc.commitment import Finalized
import time

# Initialize the argument parser
parser = argparse.ArgumentParser(description="Solana transaction script with transfer, burn, and memo instructions.")
parser.add_argument('--rpc_url', type=str, default="https://devnet.helius-rpc.com/?api-key=174a6ec2-4439-4fca-9277-b12900c71fa5", help="RPC URL to connect to Solana")
parser.add_argument('--keypair', type=str, required=True, help="Solana keypair json file")
parser.add_argument('--from_account', type=str, required=True, help="Source account public key for transfer")
parser.add_argument('--fee_account', type=str, required=True, help="Destination account public key for transfer")
parser.add_argument('--fee_amount', type=int, required=True, help="Amount to transfer in lamports")
parser.add_argument('--token_mint', type=str, required=True, help="Token mint public key for burn instruction")
parser.add_argument('--burn_account', type=str, required=True, help="Token account to burn from")
parser.add_argument('--owner_account', type=str, required=True, help="The token account owner")
parser.add_argument('--burn_amount', type=int, required=True, help="Amount of tokens to burn")
parser.add_argument('--memo_msg', type=str, required=True, help="Memo message to attach to the transaction")

args = parser.parse_args()

# Create Solana client

# http_client = Client("https://api.devnet.solana.com")
# helius = "https://devnet.helius-rpc.com/?api-key=174a6ec2-4439-4fca-9277-b12900c71fa5"
# helius = "https://mainnet.helius-rpc.com/?api-key=174a6ec2-4439-4fca-9277-b12900c71fa5"
# http_client = Client(helius)
# print("solana rpc: {}".format(args.rpc_url))
http_client = Client(args.rpc_url)

# signer = Keypair.from_bytes([143,19,100,3,213,252,67,197,16,73,65,76,72,33,185,112,213,36,134,228,109,178,41,136,115,0,195,206,234,57,232,7,39,225,234,201,93,212,181,14,64,235,184,190,89,114,47,55,181,221,51,224,26,7,109,81,138,175,218,14,149,236,48,99])
# print("Keypair: {}".format(args.keypair))
signer = Keypair.from_json(args.keypair)
# signer=Keypair.from_bytes([143,19,100,3,213,252,67,197,16,73,65,76,72,33,185,112,213,36,134,228,109,178,41,136,115,0,195,206,234,57,232,7,39,225,234,201,93,212,181,14,64,235,184,190,89,114,47,55,181,221,51,224,26,7,109,81,138,175,218,14,149,236,48,99])
# build transfer instruction
# from_account = payer.pubkey()
# fee_account = payer.pubkey()
# fee_amount = 1000
from_account = Pubkey.from_string(args.from_account)
fee_account = Pubkey.from_string(args.fee_account)
fee_amount = int(args.fee_amount)
# print("tansfer info:\n  from_account:{}\n  fee_account:{}\n  fee_amount:{}".format(from_account,fee_account,fee_amount))
transfer_ix = sp.transfer(sp.TransferParams(from_pubkey=from_account, to_pubkey=fee_account, lamports=fee_amount))

# build burn instruction
# token_mint = Pubkey.from_string("9aBWuQ5dKG7T6vAaV9WgwTxTMUs3AjfNSgDEV8Pmw4Z2")
# burn_account = Pubkey.from_string("9V5NxX9SbcXFfUZJU57QViDtXauwGrZBYfntnm54nzLA")
# owner_account = payer.pubkey()
# burn_amount = 1000
token_mint = Pubkey.from_string(args.token_mint)
burn_account = Pubkey.from_string(args.burn_account)
owner_account = Pubkey.from_string(args.owner_account)
burn_amount = int(args.burn_amount)
# print("burn info:\n  token_mint:{}\n  burn_account:{}\n  owner_account:{}\n  burn_amount:{}"
    #   .format(token_mint,burn_account,owner_account,burn_amount))
burn_ix = spl_token.burn_checked(
                spl_token.BurnCheckedParams(
                    program_id=TOKEN_PROGRAM_ID,
                    mint=token_mint,
                    account=burn_account,
                    owner=owner_account,
                    amount=burn_amount,
                    decimals=0,
                    signers=[],
                )
            )
# burn_ix = spl_token.burn(
#                 spl_token.BurnParams(
#                     program_id=TOKEN_PROGRAM_ID,
#                     account=burn_account,
#                     mint=token_mint,
#                     owner=owner_account,
#                     amount=burn_amount,
#                     signers=[],
#                 )
#             )

# build memo instruction
# memo_msg = "bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6"
memo_msg = args.memo_msg
# print("memo msg: {}".format(memo_msg))

memo_params = MemoParams(
        program_id=MEMO_PROGRAM_ID,
        signer=signer.pubkey(),
        message=bytes(memo_msg, encoding="utf8"),
    )
memo_ix = create_memo(memo_params)

ixs = [transfer_ix, burn_ix, memo_ix]

# build tx and send it to solana
blockhash = http_client.get_latest_blockhash().value.blockhash
# print("latest block hash:", blockhash)
msg = Message.new_with_blockhash(ixs, signer.pubkey(), blockhash)
txn = Transaction([signer], msg, blockhash)
opts = TxOpts(skip_confirmation=False, skip_preflight=False)
resp = http_client.send_transaction(txn)
# print("send_transaction result:", resp)

time.sleep(20)
# query signature statuse
sig_state = http_client.get_signature_statuses([resp.value])
# sig_state=http_client.confirm_transaction(resp.value).value
# print("signature status: ", sig_state)

# query tx detail
tx = http_client.get_transaction(resp.value, commitment=Finalized, encoding="jsonParsed").value
# print("tx detail: ", tx)

# output signature
print(resp.value)
