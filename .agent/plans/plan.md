1. **Repository-Aufräumung & Dateiverschiebung**
   - Das Root-Verzeichnis überprüfen. Die Datei `GEMINI.md` ist im Root-Verzeichnis nach den Regeln nicht erlaubt. Sie wird nach `.temp-archive/2026-03-08-GEMINI.md` verschoben und mittels `git rm` aus der Git-Verwaltung entfernt (da sie wiederholt im Root platziert wird).
2. **Entfernen großer, getrackter Log-Dateien**
   - Das Verzeichnis `scripts/logs/` enthält mehrere große `.log` Dateien (z.B. `mapflow.log.2026-02-26` und `mapflow.log.2026-03-05` mit jeweils über 10 MB). Diese werden aus dem Git-Tracking entfernt (`git rm --cached`) und zur Sicherheit nach `.temp-archive/` verschoben.
3. **Aktualisierung der `.gitignore`**
   - Hinzufügen von `Thumbs.db` gemäß den Standardvorgaben.
   - Hinzufügen von Einträgen für `scripts/logs/` und `mapflow.log.*`, um zukünftiges Tracking dieser großen Log-Dateien zu verhindern.
4. **Dokumentation des Aufräumvorgangs**
   - Einen Eintrag im Journal `.jules/archivist.md` hinzufügen. Dies ist notwendig, da die Platzierung von `GEMINI.md` ein wiederkehrendes Muster darstellt und das fehlerhafte Tracking der großen Log-Dateien im `scripts/logs/` Verzeichnis dokumentiert werden sollte.
5. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Führe alle notwendigen Pre-Commit-Prüfungen aus, um sicherzustellen, dass die Repository-Aufräumung keine bestehenden Tests, Formatierungen oder Projekt-Builds beeinträchtigt.
6. **Pull Request Erstellung**
   - Die Änderungen unter dem Zweig `archivist-cleanup` mit einem PR-Titel "🗂️ Archivist: Repository Cleanup" einreichen.
   - In der PR-Beschreibung die erforderlichen Abschnitte 'Was', 'Freigegeben', 'Gelöschte Dateien', 'Archivierte Dateien', 'Verschobene Dateien', und 'Umbenannte Dateien' auflisten.
