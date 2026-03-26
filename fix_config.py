import re

with open('crates/vorce-ui/src/core/config.rs', 'r') as f:
    content = f.read()

content = content.replace('Self::MapFlow(id) => write!(f, "MapFlow: {}", id),', 'Self::MapFlow(id) => write!(f, "Vorce: {}", id),')

with open('crates/vorce-ui/src/core/config.rs', 'w') as f:
    f.write(content)
