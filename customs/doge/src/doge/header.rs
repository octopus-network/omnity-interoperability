use std::{borrow::Cow, str::FromStr};

use bitcoin::{
    block::{Header, Version},
    BlockHash, CompactTarget, TxMerkleNode,
};
use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockJsonResult {
    pub hash: String,
    pub confirmations: u64,
    pub strippedsize: u64,
    pub size: u64,
    pub weight: u64,
    pub height: u64,
    pub version: i32,
    pub version_hex: String,
    pub merkleroot: String,
    pub tx: Vec<String>,
    pub time: u32,
    pub mediantime: u64,
    pub nonce: u32,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    pub previousblockhash: String,
    pub nextblockhash: String,
    // pub auxpow: Option<AuxPow>,
    // pub tx: Vec<TransactionJsonResult>,
}

impl TryFrom<BlockJsonResult> for Header {
    type Error = crate::errors::CustomsError;

    fn try_from(value: BlockJsonResult) -> Result<Self, Self::Error> {
        let header = Header {
            version: Version::from_consensus(value.version),
            prev_blockhash: BlockHash::from_str(value.previousblockhash.as_str()).map_err(|e| {
                crate::errors::CustomsError::InvalidBlockHash(
                    value.previousblockhash.clone(),
                    e.to_string(),
                )
            })?,
            merkle_root: TxMerkleNode::from_str(&value.merkleroot).map_err(|e| {
                crate::errors::CustomsError::InvalidMerkleRoot(
                    value.merkleroot.clone(),
                    e.to_string(),
                )
            })?,
            time: value.time,
            bits: CompactTarget::from_unprefixed_hex(&value.bits).map_err(|e| {
                crate::errors::CustomsError::InvalidBits(value.bits.clone(), e.to_string())
            })?,
            nonce: value.nonce,
        };

        Ok(header)
    }
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeaderJsonResult {
    pub hash: String,
    pub confirmations: i64,
    pub height: u64,
    pub version: i32,
    pub version_hex: String,
    pub merkleroot: String,
    pub time: u32,
    pub mediantime: u64,
    pub nonce: u32,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    pub previousblockhash: String,
    pub nextblockhash: Option<String>,
    pub block_header_hex: Option<String>,
}

impl Storable for BlockHeaderJsonResult {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Directive");
        dire
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl TryFrom<BlockHeaderJsonResult> for Header {
    type Error = crate::errors::CustomsError;

    fn try_from(value: BlockHeaderJsonResult) -> Result<Self, Self::Error> {
        let header = Header {
            version: Version::from_consensus(value.version),
            prev_blockhash: BlockHash::from_str(value.previousblockhash.as_str()).map_err(|e| {
                crate::errors::CustomsError::InvalidBlockHash(
                    value.previousblockhash.clone(),
                    e.to_string(),
                )
            })?,
            merkle_root: TxMerkleNode::from_str(&value.merkleroot).map_err(|e| {
                crate::errors::CustomsError::InvalidMerkleRoot(
                    value.merkleroot.clone(),
                    e.to_string(),
                )
            })?,
            time: value.time,
            bits: CompactTarget::from_unprefixed_hex(&value.bits).map_err(|e| {
                crate::errors::CustomsError::InvalidBits(value.bits.clone(), e.to_string())
            })?,
            nonce: value.nonce,
        };
        Ok(header)
    }
}

#[test]
pub fn test() {
    use bitcoin::Txid;
    let raw = r#"
    {"result":{"hash":"b44fcdcc96ded450f00f39e1971354997782dc64560590ccbe4f211b83cd2b7e","confirmations":2,"height":5571429,"version":6422788,"versionHex":"00620104","merkleroot":"8697dd9133c84f478df92d8cf5b721eaeeb15fc70e498ef15f360907320da3f8","time":1738552737,"mediantime":1738552237,"nonce":0,"bits":"1a0089df","difficulty":31151575.30981725,"chainwork":"0000000000000000000000000000000000000000000017318cbd956e24f66c5e","previousblockhash":"577fa5603f0d3d31a202312c561eab0e98c7ebaa9bd61e5ec8b6cfe4782ef414","nextblockhash":"667b7e269cbdf6a80207f562f8200dbc8012f70b05fda515064f999897651666"},"error":null,"id":1}
    "#;

    let rpc_res: crate::doge::transaction::DogeRpcResponse<BlockHeaderJsonResult> =
        serde_json::from_str(raw).unwrap();
    let result = rpc_res.try_result().unwrap();
    let block_header: Header = result.try_into().unwrap();
    // block_header.validate_pow(block_header.target()).unwrap();

    // TransactionJsonResult
    let block_raw = r#"
    {"result":{"hash":"b44fcdcc96ded450f00f39e1971354997782dc64560590ccbe4f211b83cd2b7e","confirmations":2049,"strippedsize":2687,"size":2687,"weight":10748,"height":5571429,"version":6422788,"versionHex":"00620104","merkleroot":"8697dd9133c84f478df92d8cf5b721eaeeb15fc70e498ef15f360907320da3f8","tx":["b4d7d5feab91cf7a0ad93b9b0dc6e5ea967c9e865a3a40502580c05713ab9552","264409cb182689f3a86dacdee18b9b103531bb6619082b2e6527d08868cf9d81","6676cb912793038d67a07b9773ed0d03b719271b4e28da29488e7a468349af2f","a61ed50c4948bc349bb6ad0d170e320154b6d541dae9e35394413ce60115af2f","0b9198e3e7f027c54a9d8d8aac3877776d17cf91c28e8c3de1d0733302c2471b","da7360cbb5677dac615941e8edfa1bd7776b5728b2b2663f79589689e49cea02","f088562570d5fa4dd529413a667b1fa23ecc618e314e5436a331a2076bfb243d"],"time":1738552737,"mediantime":1738552237,"nonce":0,"bits":"1a0089df","difficulty":31151575.30981725,"chainwork":"0000000000000000000000000000000000000000000017318cbd956e24f66c5e","auxpow":{"tx":{"hex":"01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff6403f0502b2cfabe6d6d091fb22b379c6292e3ea11dfbc5791b846b8710f06c40423f1897d84c99c8de520000000f09f909f092f4632506f6f6c2f620000000000000000000000000000000000000000000000000000000000000000000000050006bbe80000000000027fb64c25000000001976a914f2910ecaf7bb8d18ed71f0904e0e7456f29ce18288ac0000000000000000266a24aa21a9ed71f793873b02234e8f85653bd24b0d4732a03cf982a8557fd8a94a720ba13c0c44802342","txid":"ec7d2d2eb2e9181d7882c05f9d6312e90ca5eb7b26a203a0921d1089c0dd0b84","hash":"ec7d2d2eb2e9181d7882c05f9d6312e90ca5eb7b26a203a0921d1089c0dd0b84","size":232,"vsize":232,"version":1,"locktime":1109622852,"vin":[{"coinbase":"03f0502b2cfabe6d6d091fb22b379c6292e3ea11dfbc5791b846b8710f06c40423f1897d84c99c8de520000000f09f909f092f4632506f6f6c2f620000000000000000000000000000000000000000000000000000000000000000000000050006bbe800","sequence":0}],"vout":[{"value":6.25784447,"n":0,"scriptPubKey":{"asm":"OP_DUP OP_HASH160 f2910ecaf7bb8d18ed71f0904e0e7456f29ce182 OP_EQUALVERIFY OP_CHECKSIG","hex":"76a914f2910ecaf7bb8d18ed71f0904e0e7456f29ce18288ac","reqSigs":1,"type":"pubkeyhash","addresses":["DTFfqUPJctvKQuGCXkpL2sW99EXftxgPsF"]}},{"value":0,"n":1,"scriptPubKey":{"asm":"OP_RETURN aa21a9ed71f793873b02234e8f85653bd24b0d4732a03cf982a8557fd8a94a720ba13c0c","hex":"6a24aa21a9ed71f793873b02234e8f85653bd24b0d4732a03cf982a8557fd8a94a720ba13c0c","type":"nulldata"}}],"blockhash":"c2b901e6f829e4161cbdfe04368d340a0196942302c54656cd0eec2f2c8e1e6d"},"index":0,"chainindex":8,"merklebranch":["58bb37aa18d56c0da7f112543032eee5d26fdb0371216b3f37bac9902ca27431","323f8e0bdff411aa67c0bc9fb02339a1b95a285ef2a17ca5f117e9f1bc57015f","5b338cd0b497d4ce8b9cfa6243b48f74be7265265efc40b6f4a046cbc5745de9","29c10bc93cb47b85afeec0ff3fc1004cb68fd10ea3c074cbf8ab3015a006e3d0","46c7f2320273babedf2fbe9ed622a1c18577a1385e73b4dd90303b12f531e820","70bd9ec2a1083291e7c5031f5d751c191362ac011d342d152536456768e69066","c0258138654da2b53ad29d6f442cd5f94dd1064682eba8245615c228cd04d719","f043768fc08e441ed562fdd9258c7a96dbd93161b02f81c0c8f68580646b920b","55250673376a0c30e0e4a77b4086f11195cdab68424b5e4247b92ef45471551b"],"chainmerklebranch":["0000000000000000000000000000000000000000000000000000000000000009","c8a756a6441def5a7734cba02852a3c8d97545452183dd00a207f42d3f1b0b71","b58ef4f10a5fed631fac991c136512e0e95b37f80d58d4a9498d82c13ae3125c","9f106384713e60ad08ae99dd67653ab17d16bbc054d7c10be38544d3683a1c9f","4174f45c39fb6684a6710c04312f8fe8093d44714b858cc47fb9e39f27a890ea"],"parentblock":"14000020250936f4a4a548ac27a19c57718ddd4a0d2f2fa6e7b94e7cfdcf9bd97a51ef6708a07f78e003591bff278d0faccc11d4055054e8ee7f152dabd761aaac71df0fd135a06750bc39190c6c094e"},"previousblockhash":"577fa5603f0d3d31a202312c561eab0e98c7ebaa9bd61e5ec8b6cfe4782ef414","nextblockhash":"667b7e269cbdf6a80207f562f8200dbc8012f70b05fda515064f999897651666"},"error":null,"id":1}
    "#;
    let block_rpc_res: crate::doge::transaction::DogeRpcResponse<BlockJsonResult> =
        serde_json::from_str(block_raw).unwrap();
    let block_json = block_rpc_res.try_result().unwrap();
    let txids: Vec<bitcoin::Txid> = block_json
        .tx
        .iter()
        .map(|hex| hex.parse::<Txid>().unwrap())
        .collect();
    // let txids: Vec<Txid> = [
    //     "b047cea044a471942e9f07e3950c1e395f1e0f654b7522e20c269c5afad2580b",
    //     "3eda4a077d0ee3bf3861b238905e49001f85e30e0a662cc4455f23b577be3d6a",
    //     "d25b0de33d55329ab32cd2fa75eb1636cb150307da5b816402bb463d54b46d5e",
    //     "980d8ab7b94efc69d7105f2ea419d67735a7ff6a58d133729a5127f199ca3f72",
    //     "7592862e7d4c0001a504e0d6e2b874a7f18dee0b482e1aca9fe0864dba2cb1c1",
    //     "6e2a9b84e7c3fd14f3ad142bbaa9cb357a549e3351fc3e91b68f3c43eda3f06a",
    //     "ede2f2ccbf77a04620c3639c7540a7cf89bba5be0f39e2141da2ce1b991990ce",
    //     "bc60c18fe96f9d49a44d1ce01f884b8863984a8aee52aad305bd21cb2b160ef6",
    //     "05cf587f02b1c355b45c27149b64d243045e03e3fbbb5470f1cc186ccc3a42b1",
    //     "07fdfb891139c681503276867a682c9c5aeac28fdef13c940acfe9ce3f14d0b1",
    //     "f293e079d91efe06c9386be27a9f8df199f3e3a529ea935a2bda8c0f65d7304b",
    //     "ee941f8ea26411ae830c827f99d3416359f573bf43bf7fead011aafd0eb268f1",
    //     "11315c5ab612a6a90db6776a2c5d048b2efa1a1954a3de0b52908c33828b135d",
    //     "b160f41718d268a2635e666e926287b09e60fcfa91221ea93619653624cc4c48",
    //     "fc524a3042e27bcc16288e1823c7a8d08c7d80f094ae1af9a7b9f27183f72563",
    //     "d2e14218cda2fb17c5f64ae0331002776eeeaa5543cb77149d5c856d6ae9648e",
    //     "7577f8d84e03def7293596975f3bc9a9e954239cd7b32143ef376859783a6a5d",
    //     "a03aabc5efe1252b0b5dd2fb242d59d6582e4be4a426075cfdbe91210c6366f1",
    //     "a50e1e3d21e0d7559eeb2caa2578bf09b3dc3bf42487e82168f00f79a461a844"
    // ]
    // .iter()
    // .map(|hex| hex.parse::<Txid>().unwrap())
    // .collect();

    let aim_txid = "7592862e7d4c0001a504e0d6e2b874a7f18dee0b482e1aca9fe0864dba2cb1c1";
    let merkle_block = bitcoin::MerkleBlock::from_header_txids_with_predicate(
        &block_header,
        txids.as_slice(),
        |t| t.to_string().eq(aim_txid),
    );

    assert_eq!(merkle_block.header.block_hash(), block_header.block_hash());

    dbg!(&block_header);
}
