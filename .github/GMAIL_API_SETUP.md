# Gmail API Setup für CI-12

Diese Anleitung erklärt, wie du die Gmail API Credentials für den CI-12 Workflow erstellst.

## Voraussetzungen

- Google Account (mit Zugriff auf die Jules-Benachrichtigungs-Emails)
- Zugriff auf [Google Cloud Console](https://console.cloud.google.com)

## Schritt 1: Google Cloud Project erstellen

1. Öffne https://console.cloud.google.com
2. Erstelle ein neues Projekt (z.B. "MapFlow-CI")
3. Wähle das Projekt aus

## Schritt 2: Gmail API aktivieren

1. Gehe zu **APIs & Services** → **Enable APIs and Services**
2. Suche nach "Gmail API"
3. Klicke auf **Enable**

## Schritt 3: OAuth 2.0 Credentials erstellen

1. Gehe zu **APIs & Services** → **Credentials**
2. Klicke auf **Create Credentials** → **OAuth client ID**
3. Falls nötig: Konfiguriere den OAuth Consent Screen:
   - User Type: **External**
   - App Name: "MapFlow CI"
   - Scopes: `gmail.readonly`, `gmail.modify`
4. Erstelle OAuth Client ID:
   - Application type: **Desktop app**
   - Name: "MapFlow GitHub Actions"
5. Notiere: **Client ID** und **Client Secret**

## Schritt 4: Refresh Token generieren

Du kannst den [OAuth 2.0 Playground](https://developers.google.com/oauthplayground/) nutzen:

1. Öffne https://developers.google.com/oauthplayground/
2. Klicke auf das Zahnrad (Settings) → **Use your own OAuth credentials**
3. Trage Client ID und Client Secret ein
4. Wähle Scopes:
   - `https://www.googleapis.com/auth/gmail.readonly`
   - `https://www.googleapis.com/auth/gmail.modify`
5. Klicke **Authorize APIs** → Melde dich an
6. Klicke **Exchange authorization code for tokens**
7. Kopiere den **Refresh Token**

## Schritt 5: GitHub Secrets hinzufügen

Gehe zu deinem GitHub Repository:
**Settings** → **Secrets and variables** → **Actions** → **New repository secret**

Erstelle diese Secrets:

| Secret Name | Wert |
|------------|------|
| `GMAIL_CLIENT_ID` | Deine Client ID |
| `GMAIL_CLIENT_SECRET` | Dein Client Secret |
| `GMAIL_REFRESH_TOKEN` | Dein Refresh Token |

## Testen

1. Führe den Workflow manuell aus: **Actions** → **CI-12: Gmail Jules Monitor** → **Run workflow**
2. Aktiviere Debug-Modus für mehr Details
3. Prüfe die Logs auf Fehler

## Fehlerbehebung

### "invalid_grant" Fehler
- Der Refresh Token kann ablaufen wenn die App noch nicht verifiziert ist
- Lösung: Generiere einen neuen Refresh Token

### "Token has been expired or revoked"
- Der Token wurde widerrufen oder ist abgelaufen
- Lösung: Generiere einen neuen Refresh Token im OAuth Playground

### Keine Emails gefunden
- Stelle sicher, dass Jules-Emails an die Gmail-Adresse gesendet werden
- Prüfe den Email-Filter Query in den Workflow-Logs
