use cosmwasm_std::Order;

pub fn option_bool_to_order(descending: Option<bool>) -> Order {
    match descending {
        Some(_descending) => {
            if _descending {
                Order::Descending
            } else {
                Order::Ascending
            }
        }
        _ => Order::Ascending,
    }
}
