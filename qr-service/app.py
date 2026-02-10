# QR servis - generise QR kodove za karte

import io
import qrcode
from flask import Flask, request, jsonify, send_file

app = Flask(__name__)


@app.route("/health", methods=["GET"])
def health():
    return jsonify({"service": "qr-service", "status": "ok"})


@app.route("/qr", methods=["POST"])
def generate_qr():
    """
    Prima JSON sa podacima o karti i vraca QR kod kao PNG sliku.
    Body: { "ticket_code": "TKT-A1B2C3D4", "event_title": "...", "user_email": "..." }
    """
    data = request.get_json()

    if not data or "ticket_code" not in data:
        return jsonify({"error": "ticket_code je obavezan"}), 400

    # tekst sadrzi sve bitne podatke o karti
    qr_content = f"TICKET:{data['ticket_code']}"

    if "event_title" in data:
        qr_content += f"|EVENT:{data['event_title']}"

    if "user_email" in data:
        qr_content += f"|USER:{data['user_email']}"

    # Generisemo QR kod
    qr = qrcode.QRCode(version=1, box_size=10, border=4)
    qr.add_data(qr_content)
    qr.make(fit=True)

    img = qr.make_image(fill_color="black", back_color="white")

    # Pretvaramo sliku u bytes i vracamo kao PNG
    buffer = io.BytesIO()
    img.save(buffer, format="PNG")
    buffer.seek(0)

    return send_file(buffer, mimetype="image/png", download_name=f"{data['ticket_code']}.png")


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=3005)
