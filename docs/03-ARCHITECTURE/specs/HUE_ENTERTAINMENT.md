# Spezifikation: Philips Hue Entertainment Integration

## 1. Übersicht

Ziel ist die Integration von Philips Hue Leuchten in MapFlow. Anders als bei einfachen Smart-Home-Apps nutzen wir nicht die langsame REST-API (Polling), sondern die **Hue Entertainment API v2** mit **DTLS (Datagram Transport Layer Security)** Streaming. Das ermöglicht extrem niedrige Latenz (~20ms) und hohe Update-Raten (25-50 Hz) für perfekte Synchronisation mit Musik und Video.

## 2. Technische Grundlagen

### 2.1 Protokoll & API

* **Discovery**: mDNS (ZeroConf) zum Finden der Bridge im Netzwerk.
* **Authentifizierung**: App-Registrierung per Button-Press auf der Bridge (Whitelisting).
* **Konfiguration**: Abruf der "Entertainment Areas" (Räumliche Positionierung der Lampen) via REST/HTTPS.
* **Streaming**: DTLS auf Port 2100.
  * Format: Proprietäres Philips-Format (kein offenes Art-Net/DMX, aber gut dokumentiert).
  * Farbraum: CIE xy (nicht RGB!), Umrechnung erforderlich.

### 2.2 Rust Stack

* **HTTP/REST**: `reqwest` (vorhanden).
* **Discovery**: `mdns-sd` crate.
* **DTLS**: `dtls` crate (reine Rust-Implementierung des Webrtc-Projekts) oder `openssl` bindings (schwieriger zu builden). Alternativ: Simple UDP, falls Handshake manuell implementiert wird (DTLS PSK ist der kritische Teil).
* **Farb-Konvertierung**: `palette` crate (für RGB -> CIE xy Konvertierung).

## 3. Architektur-Integration in MapFlow

### 3.1 Modul-Struktur

Neues Modul in `mapmap-control`: `crates/mapmap-control/src/hue/`

* `bridge.rs`: Discovery und Auth-Flow.
* `stream.rs`: Der DTLS-Client und Loop.
* `mapping.rs`: Konvertierung von MapFlow-Farben (RGB) in Hue-Commands.

### 3.2 Workflow (User Experience)

1. **Setup**: User geht in Settings -> "Lighting".
2. **Pairing**: Klick auf "Connect Bridge" -> User muss physikalischen Button auf der Hue Bridge drücken. -> App speichert `username` und `client_key`.
3. **Area Selection**: MapFlow lädt die in der Hue App konfigurierten "Entertainment Areas" (z.B. "Wohnzimmer", "DJ Booth").
4. **Mapping**:
    * **Modus A (Ambient)**: Durchschnittsfarbe des gesamten Video-Outputs wird auf alle Lampen gesendet.
    * **Modus B (Spatial)**: Die Position der Lampen im Raum (aus Hue App) wird auf den MapFlow-Canvas gemappt. Lampe links leuchtet, wenn links im Video etwas passiert.
    * **Modus C (Strobe)**: Audio-Reactive Trigger (Kick-Drum) lässt alle Lampen weiß blitzen.

## 4. Technische Herausforderungen

### 4.1 Farbraum-Konvertierung

LEDs haben einen begrenzten Gamut. Einfaches RGB senden führt oft zu falschen Farben (besonders Grün und Cyan).

* **Lösung**: RGB -> XYZ -> CIE xy Konvertierung mit Gamut-Correction für das jeweilige Lampen-Modell (Gamut A, B, C).

### 4.2 Keep-Alive

Der DTLS-Kanal wird von der Bridge geschlossen, wenn keine Daten kommen.

* **Lösung**: Heartbeat-Pakete senden, auch wenn schwarz (= aus).

### 4.3 Performance

Wir dürfen den Main-Thread nicht blockieren. Der Hue-Streamer muss in einem eigenen Thread (`std::thread` oder `tokio::spawn`) laufen und via Channel (`crossbeam`) Updates vom Renderer erhalten.

## 5. Implementierungs-Phasen

1. **Phase 1**: Discovery & Pairing Tool (CLI).
2. **Phase 2**: Einfacher UDP-Stream (Test ohne DTLS oft nicht möglich, daher direkt DTLS).
3. **Phase 3**: Integration in Renderer (Color Sampling).
4. **Phase 4**: UI für Area-Auswahl.
