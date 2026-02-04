import sys

def resolve_conflict(input_path, output_path):
    with open(input_path, 'r') as f:
        lines = f.readlines()

    with open(output_path, 'w') as f:
        mode = "normal" # normal, head, skip
        for line in lines:
            if line.strip().startswith("<<<<<<< HEAD"):
                mode = "head"
                continue
            elif line.strip().startswith("======="):
                mode = "skip"
                continue
            elif line.strip().startswith(">>>>>>>"):
                mode = "normal"
                continue

            if mode == "normal" or mode == "head":
                f.write(line)

if __name__ == "__main__":
    resolve_conflict("crates/mapmap-ui/src/editors/module_canvas.rs", "fixed_canvas.rs")
