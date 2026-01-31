# 📓 LINA STYLEUI JOURNAL

## 2024-05-23 👁 Initial Observation
**Learning:** The current `LayerPanel` uses `ui.group()` for every layer, creating a "boxed" look that wastes space and adds visual noise ("box-in-box" syndrome). Professional VJ software (Resolume, HeavyM) typically uses flat, alternating-color rows for lists to maximize density and readability.
**Action:** Replace `ui.group()` with flat, striped rows. Use selection highlights (Mint/Cyan) to indicate active layer instead of a box.
