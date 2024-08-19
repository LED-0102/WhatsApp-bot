#[derive(Debug, Clone)]
pub enum WorkflowStage {
    Greeting,
    ProductName,
    PurchaseDate,
    WarrantyDuration,
    CustomerName,
    SerialNumber,
    AdditionalWarrantyTerms,
    WarrantyCard
}

impl Default for WorkflowStage {
    fn default() -> Self {
        WorkflowStage::Greeting
    }
}