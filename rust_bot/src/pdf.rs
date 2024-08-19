use serde::Serialize;
use crate::user_state::UserState;

#[derive(Debug, Serialize)]
pub struct PDFInfo<'a> {
    pub product_name: &'a str,
    pub purchase_date: &'a str,
    pub warranty_duration: &'a str,
    pub customer_name: &'a str,
    pub serial_number: &'a str,
    pub additional_terms: &'a str,
    pub from: &'a str,
}

