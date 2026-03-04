sed -i 's|docs/01-GETTING-STARTED/|docs/user/getting-started/|g' crates/mapmap/README.md
sed -i 's|docs/02-USER-GUIDE/|docs/user/manual/|g' crates/mapmap/README.md
sed -i 's|docs/03-ARCHITECTURE/|docs/dev/architecture/|g' crates/mapmap/README.md

sed -i 's|docs/01-OVERVIEW/README.md|docs/user/getting-started/README.md|g' docs/project/audits/DOCUMENTATION_AUDIT.md
sed -i 's|docs/02-USER-GUIDE/|docs/user/manual/|g' docs/project/audits/DOCUMENTATION_AUDIT.md
sed -i 's|04-USER-GUIDE|docs/user/manual|g' docs/project/audits/DOCUMENTATION_AUDIT.md
sed -i 's|docs/03-ARCHITECTURE|docs/dev/architecture|g' docs/project/audits/DOCUMENTATION_AUDIT.md
