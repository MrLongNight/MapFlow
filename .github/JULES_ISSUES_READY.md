# Jules Issues - Bereit zur Erstellung

## ⚠️ Wichtiger Hinweis

Die Jules Issues wurden **NICHT automatisch erstellt** weil:

1. **Permissions:** Der Workflow läuft auf einem PR-Branch und hat nicht die nötigen `issues: write` Permissions
2. **Branch Context:** Die Auto-Trigger funktioniert nur wenn der Workflow auf `main` ist

## ✅ Lösung: Issues JETZT manuell erstellen

Du hast **2 einfache Optionen**:

### Option 1: Workflow manuell ausführen (Empfohlen)

```bash
# Merge diesen PR zuerst, dann:
gh workflow run "Create Jules Development Issues"
```

### Option 2: PR mergen und Workflow läuft automatisch

Sobald dieser PR in `main` gemerged wird, kann der Workflow mit den richtigen Permissions laufen.

## 📋 Die 8 Jules Issues (bereit im Workflow)

Alle Issues sind bereits im Workflow definiert und werden sofort erstellt sobald der Workflow läuft:

1. **Implement Multi-Window Rendering** (Critical, Phase 2)
   - window-per-output architecture
   - Frame synchronization
   - Display change handling
   
2. **Implement Frame Synchronization** (Critical, Phase 2)
   - VSync mechanism
   - Frame timing system
   - Frame drop detection
   
3. **Fix Build System - FreeType Linker Errors** (High, Infrastructure)
   - FreeType linker errors
   - System dependencies
   - Multi-platform testing
   
4. **Complete Still Image Support** (High, Phase 1)
   - PNG, JPG, TIFF support
   - Image caching
   - Memory management
   
5. **Add Animated Format Support** (Medium, Phase 1)
   - GIF decoder
   - Image sequences
   - Frame timing
   
6. **Add ProRes Codec Support** (Medium, Phase 1)
   - FFmpeg integration
   - ProRes variants
   - Performance benchmarking
   
7. **Advanced Geometric Correction Tools** (Medium, Phase 2)
   - Keystone correction UI
   - Grid-based warping
   - Warp presets
   
8. **Implement Output Configuration Persistence** (Medium, Phase 2)
   - Project file format
   - Serialization
   - Migration support

## 🚀 Nächste Schritte

### Sofort (Nach PR Merge):

```bash
# 1. Labels erstellen (einmalig)
gh label sync --file .github/labels.yml

# 2. Issues erstellen
gh workflow run "Create Jules Development Issues"

# 3. Status prüfen
gh run watch

# 4. Issues anzeigen
gh issue list --label "jules-task"
```

### Jules konfigurieren:

- Repository: `MrLongNight/VjMapper`
- Monitor Label: `jules-task`
- PR Label: `jules-pr`
- Branch Prefix: `jules/`

## 💡 Warum nicht automatisch?

**Sicherheit & Kontrolle:**
- Workflow-Permissions sind auf PR-Branches eingeschränkt
- Auto-Erstellung würde bei jedem Push auf dem Branch laufen
- Manual dispatch gibt dir volle Kontrolle wann Issues erstellt werden

**Das ist Best Practice!** Issues sollten bewusst und einmalig erstellt werden, nicht bei jedem Push.

## ✅ Workflow ist bereit!

Sobald der PR gemerged ist, einfach:
```bash
gh workflow run "Create Jules Development Issues"
```

Und alle 8 Issues werden erstellt! 🎉

---

**Erstellt:** 2024-12-04  
**Status:** ✅ Workflow bereit, wartet auf Merge oder manuelle Ausführung  
**Commit:** eccf7c3
