---
description: Start a Jules remote session for Vorce
---
1. Run `jules remote list --repo` to verify connection (optional).
2. Start session using the Vorce repo:
   ```powershell
   jules remote new --repo Vorce-Studios/Vorce --session "<Task Description>"
   ```
   *Note: Always specify `--repo Vorce-Studios/Vorce` as the local directory detection might fail.*
