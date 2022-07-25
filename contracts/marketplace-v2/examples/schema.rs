use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use marketplace_v2::{MarketplaceContract, msg};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(MarketplaceContract), &out_dir);
    export_schema(&schema_for!(msg::InstantiateMsg), &out_dir);
    export_schema(&schema_for!(msg::ExecuteMsg), &out_dir);
    export_schema(&schema_for!(msg::QueryMsg), &out_dir);

    export_schema(&schema_for!(msg::QueryOptions<msg::AskExpiryOffset>), &out_dir);
    export_schema(&schema_for!(msg::QueryOptions<msg::AskPriceOffset>), &out_dir);
    export_schema(&schema_for!(msg::QueryOptions<msg::BidExpiryOffset>), &out_dir);
    export_schema(&schema_for!(msg::QueryOptions<msg::BidTokenPriceOffset>), &out_dir);
    export_schema(&schema_for!(msg::CollectionBidPriceOffset), &out_dir);
    export_schema(&schema_for!(msg::CollectionBidExpiryOffset), &out_dir);
    export_schema(&schema_for!(msg::AskResponse), &out_dir);
    export_schema(&schema_for!(msg::AsksResponse), &out_dir);
    export_schema(&schema_for!(msg::AskCountResponse), &out_dir);
    export_schema(&schema_for!(msg::BidResponse), &out_dir);
    export_schema(&schema_for!(msg::BidsResponse), &out_dir);
    export_schema(&schema_for!(msg::ConfigResponse), &out_dir);
    export_schema(&schema_for!(msg::CollectionBidResponse), &out_dir);
    export_schema(&schema_for!(msg::CollectionBidsResponse), &out_dir);
}