use std::collections::HashMap;
use reqwest::Client;
use serde_json::Value;
use crate::config::Config;
use crate::pdf::PDFInfo;
use crate::workflow::WorkflowStage;

#[derive(Debug, Default)]
pub struct UserState {
    stage: WorkflowStage,
    product_name: Option<String>,
    purchase_date: Option<String>,
    warranty_duration: Option<String>,
    customer_name: Option<String>,
    serial_number: Option<String>,
    additional_terms: Option<String>,
    previous_state: PreviousState
}

#[derive(Debug, Default)]
pub struct PreviousState {
    product_name: Option<String>,
    purchase_date: Option<String>,
    warranty_duration: Option<String>,
    customer_name: Option<String>,
    serial_number: Option<String>,
    additional_terms: Option<String>,
}

impl UserState {
    fn get_message(&self) -> String {
        match &self.stage {
            WorkflowStage::Greeting => "Welcome to the Warranty Card Generator Bot! How can I assist you today?\n1. Create New Warranty Card\n2. View Last Warranty Card".to_string(),
            WorkflowStage::ProductName => "Please enter the product name.".to_string(),
            WorkflowStage::PurchaseDate => "Please enter the purchase date (e.g., YYYY-MM-DD).".to_string(),
            WorkflowStage::WarrantyDuration => "Please enter the warranty duration (e.g., 1 year, 2 years).".to_string(),
            WorkflowStage::CustomerName=> "Please enter the customer's name.".to_string(),
            WorkflowStage::SerialNumber => "Please enter the serial number (optional) or type 'none'.".to_string(),
            WorkflowStage::AdditionalWarrantyTerms=> "Please enter any additional warranty terms (optional) or type 'none'.".to_string(),
            WorkflowStage::WarrantyCard => {
                let product_name = self.product_name.as_deref().unwrap_or("None");
                let purchase_date = self.purchase_date.as_deref().unwrap_or("None");
                let warranty_duration = self.warranty_duration.as_deref().unwrap_or("None");
                format!(
                    "Generating your warranty card for {} purchased on {} with a {} warranty...",
                    product_name, purchase_date, warranty_duration
                )
            }
        }
    }

    pub(crate) async fn process_message(&mut self, body: &str, from: &str) {
        if body == "/reset" {
            self.reset_state();
        }
        match &mut self.stage {
            WorkflowStage::Greeting => {
                if body == "Hi" {
                    let message = self.get_message();
                    self.message(from, &message, true).await;
                } else {
                    let message = "Invalid response. Valid responses ['Hi']";
                    self.message(from, &message, false).await;
                }
            }
            WorkflowStage::ProductName => {
                if body == "1" {
                    let message = self.get_message();
                    self.message(from, &message, true).await;
                } else if body == "2" {
                    self.send_pdf(from, false).await.unwrap();
                } else {
                    let message = "Invalid response. Valid responses ['1', '2']";
                    self.message(from, &message, false).await;
                }
            }
            WorkflowStage::PurchaseDate => {
                self.product_name = Some(body.parse().unwrap());
                let message = self.get_message();
                self.message(from, &message, true).await;
            }
            WorkflowStage::WarrantyDuration => {
                self.purchase_date = Some(body.parse().unwrap());
                let message = self.get_message();
                self.message(from, &message, true).await;
            }
            WorkflowStage::CustomerName => {
                self.warranty_duration = Some(body.parse().unwrap());
                let message = self.get_message();
                self.message(from, &message, true).await;
            }
            WorkflowStage::SerialNumber => {
                self.customer_name = Some(body.parse().unwrap());
                let message = self.get_message();
                self.message(from, &message, true).await;
            }
            WorkflowStage::AdditionalWarrantyTerms => {
                self.serial_number = Some(body.parse().unwrap());
                let message = self.get_message();
                self.message(from, &message, true).await;
            }
            WorkflowStage::WarrantyCard => {
                self.additional_terms = Some(body.parse().unwrap());
                let message = self.get_message();
                self.message(from, &message, true).await;

                self.send_pdf(from, true).await.unwrap();

                self.reset_previous();
            }
        }
    }

    async fn send_message (&self, to: &str, body: &str, media_url: Option<String>) -> Result<(), String> {
        let config = Config::new();
        let auth_token = config.map.get("AUTH_TOKEN").unwrap().as_str().unwrap();
        let from = config.map.get("FROM").unwrap().as_str().unwrap();
        let account_sid = config.map.get("ACCOUNT_SID").unwrap().as_str().unwrap();

        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            account_sid
        );

        let mut params = HashMap::new();
        params.insert("To", to.to_string());
        params.insert("From", from.to_string());
        params.insert("Body", body.to_string());

        if let Some(url) = media_url {
            println!("Media URL {}", &url);
            params.insert("MediaUrl", url);
        }

        let encoded_params = serde_urlencoded::to_string(&params).map_err(|e| e.to_string())?;
        let client = Client::new();

        let response = client
            .post(&url)
            .basic_auth(account_sid, Some(auth_token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(encoded_params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(String::from("Error sending message"))
        }
    }

    async fn send_pdf (&self, from: &str, toggle: bool) -> Result<(), String> {
        let config = Config::new();
        let service_url = config.map.get("PDF_SERVICE").unwrap().as_str().unwrap();
        let mut pdf_info = PDFInfo {
            product_name: self.product_name.as_deref().unwrap_or("Product Name"),
            purchase_date: self.purchase_date.as_deref().unwrap_or("XX-XX-XXXX"),
            warranty_duration: self.warranty_duration.as_deref().unwrap_or("X Years"),
            customer_name: self.customer_name.as_deref().unwrap_or("Customer Name"),
            serial_number: self.serial_number.as_deref().unwrap_or("S No"),
            additional_terms: self.additional_terms.as_deref().unwrap_or("None"),
            from,
        };

        if !toggle {
            pdf_info = PDFInfo {
                product_name: self.previous_state.product_name.as_deref().unwrap_or("Product Name"),
                purchase_date: self.previous_state.purchase_date.as_deref().unwrap_or("XX-XX-XXXX"),
                warranty_duration: self.previous_state.warranty_duration.as_deref().unwrap_or("X Years"),
                customer_name: self.previous_state.customer_name.as_deref().unwrap_or("Customer Name"),
                serial_number: self.previous_state.serial_number.as_deref().unwrap_or("S No"),
                additional_terms: self.previous_state.additional_terms.as_deref().unwrap_or("None"),
                from,
            };
        }

        let client = Client::new();
        let pdf_url = &format!("{}/generate_pdf", service_url);
        let response = client
            .post(pdf_url)
            .json(&pdf_info)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let response_body: Value = resp.json().await.unwrap_or_default();
                    let mut download_url = response_body["download_url"].as_str();
                    let download_service = config.map.get("DOWNLOAD_SERVICE").unwrap().as_str().unwrap();
                    let final_url = match download_url {
                        Some(url) => Option::from(format!("{}{}", download_service, url)),
                        None => None,
                    };
                    self.send_message(from, "Warranty Card", final_url).await?;
                    Ok(())
                } else {
                    Err("Internal Server Error".to_string())
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    async fn message (&mut self, from: &str, message: &str, toggle: bool) {
        match self.send_message(from, message, None).await {
            Ok(_) => {
                if toggle == true {self.next_state();}
            }
            Err(e) => {
                println!("Message {message} could not be sent! Error {e}");
            }
        }
    }

    fn reset_previous (&mut self) {
        self.previous_state.product_name = self.product_name.take();
        self.previous_state.purchase_date = self.purchase_date.take();
        self.previous_state.warranty_duration = self.warranty_duration.take();
        self.previous_state.customer_name = self.customer_name.take();
        self.previous_state.serial_number = self.serial_number.take();
        self.previous_state.additional_terms = self.additional_terms.take();
        self.stage = WorkflowStage::default();
    }

    fn next_state (&mut self) {
        match &self.stage {
            WorkflowStage::Greeting => {
                self.stage = WorkflowStage::ProductName;
            }
            WorkflowStage::ProductName => {
                self.stage = WorkflowStage::PurchaseDate;
            }
            WorkflowStage::PurchaseDate => {
                self.stage = WorkflowStage::WarrantyDuration;
            }
            WorkflowStage::WarrantyDuration => {
                self.stage = WorkflowStage::CustomerName;
            }
            WorkflowStage::CustomerName => {
                self.stage = WorkflowStage::SerialNumber;
            }
            WorkflowStage::SerialNumber => {
                self.stage = WorkflowStage::AdditionalWarrantyTerms;
            }
            WorkflowStage::AdditionalWarrantyTerms => {
                self.stage = WorkflowStage::WarrantyCard;
            }
            WorkflowStage::WarrantyCard => {
                self.stage = WorkflowStage::Greeting;
            }
        }
    }

    fn reset_state (&mut self) {
        self.stage = WorkflowStage::default();
    }
}