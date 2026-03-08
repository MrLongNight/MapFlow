import os

with open("crates/mapmap/src/orchestration/evaluation.rs", "r") as f:
    content = f.read()

bad_loop = """                for i in 0..9.min(analysis.band_energies.len()) {
                    b[i] = analysis.band_energies[i];
                }"""

good_loop = """                let len = 9.min(analysis.band_energies.len());
                b[..len].copy_from_slice(&analysis.band_energies[..len]);"""

content = content.replace(bad_loop, good_loop)

with open("crates/mapmap/src/orchestration/evaluation.rs", "w") as f:
    f.write(content)
