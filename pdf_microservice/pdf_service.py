from flask import Flask, request, Response, send_from_directory, jsonify
from reportlab.lib.pagesizes import letter
from reportlab.pdfgen import canvas
from reportlab.lib.units import inch
from io import BytesIO
import os
import uuid

app = Flask(__name__)

# Pre-stored company details
COMPANY_LOGO_PATH = 'download.jpg'
COMPANY_CONTACT_INFO = "Company Contact Information: 123-456-7890"

# Directory to save generated PDFs
PDF_DIR = 'generated_pdfs'
os.makedirs(PDF_DIR, exist_ok=True)

@app.route('/generate_pdf', methods=['POST'])
def generate_pdf():
    # Get JSON data from the request
    data = request.get_json()

    # Extract details from JSON
    product_name = data.get('product_name', 'Unknown Product')
    purchase_date = data.get('purchase_date', 'Unknown Date')
    warranty_duration = data.get('warranty_duration', 'Unknown Duration')
    customer_name = data.get('customer_name', 'Unknown Customer')
    serial_number = data.get('serial_number', 'N/A')
    additional_terms = data.get('additional_terms', 'None')
    user_contact = data.get('from', 'None')

    # Create a unique filename for the PDF
    pdf_filename = f"{user_contact}_{uuid.uuid4()}.pdf"
    pdf_path = os.path.join(PDF_DIR, pdf_filename)

    # Create a canvas object
    c = canvas.Canvas(pdf_path, pagesize=letter)
    width, height = letter

    # Draw company logo
    if os.path.exists(COMPANY_LOGO_PATH):
        c.drawImage(COMPANY_LOGO_PATH, 0.5 * inch, height - 1.5 * inch, width=2 * inch, height=1 * inch)

    # Add some padding below the logo
    y_position = height - 2 * inch

    # Draw text details
    c.drawString(1 * inch, y_position, f"Product Name: {product_name}")
    c.drawString(1 * inch, y_position - 20, f"Purchase Date: {purchase_date}")
    c.drawString(1 * inch, y_position - 40, f"Warranty Duration: {warranty_duration}")
    c.drawString(1 * inch, y_position - 60, f"Customer Name: {customer_name}")
    c.drawString(1 * inch, y_position - 80, f"Serial Number: {serial_number}")
    c.drawString(1 * inch, y_position - 100, f"Additional Terms: {additional_terms}")
    c.drawString(1 * inch, y_position - 120, COMPANY_CONTACT_INFO)

    # Save the PDF
    c.save()

    # Generate download URL
    download_url = f"/{pdf_filename}"

    # Return the download URL to the client
    return jsonify({"download_url": download_url}), 201

if __name__ == '__main__':
    app.run(debug=True)
