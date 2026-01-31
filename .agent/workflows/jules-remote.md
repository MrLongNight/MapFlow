---
description: Start a Jules remote session for MapFlow
---
1. Run `jules remote list --repo` to verify connection (optional).
2. Start session using the MapFlow repo:
   ```powershell
   jules remote new --repo MrLongNight/MapFlow --session "<Task Description>"
   ```
   *Note: Always specify `--repo MrLongNight/MapFlow` as the local directory detection might fail.*
