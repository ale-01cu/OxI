import os
import sys

# ------------------------------------------------------------------
# CONFIGURATION
# ------------------------------------------------------------------
ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_FILE = "consolidated_code.min.txt"

ALLOWED_EXTENSIONS = ('.rs', '.config', ".ts", ".tsx", ".json", ".mjs", ".js")

EXCLUDED = [
    "node_modules", ".git", ".next", "dist", "build", ".cache", "coverage",
    ".vscode", ".idea", "consolidated_code.txt", "consolidated_code.min.txt",
    "consolidate.js", "consolidate.py", ".env", ".env.local",
    ".env.production", ".env.development", "package-lock.json", "yarn.lock",
    "pnpm-lock.yaml", "pnpm-workspace.yaml", "public", ".dockerignore",
    ".gitignore", "TODO.md", 'Libraries', 'packages', 'bin', 'target'
]

MAX_UNIFIED_SIZE = 1 * 1024 * 1024          # 1 MB
MAX_CHUNK_SIZE   = 512 * 1024               # 512 KB
# ------------------------------------------------------------------

def is_source_file(filename):
    return filename.endswith(ALLOWED_EXTENSIONS)

def should_exclude(file_or_dir_name, full_path):
    if file_or_dir_name in EXCLUDED:
        return True
    for part in full_path.split(os.sep):
        if part in EXCLUDED:
            return True
    return False

def minify_content(content):
    lines = content.splitlines()
    minified_lines = [line.strip() for line in lines if line.strip()]
    return "\n".join(minified_lines)

def write_file_to_output(file_path, out_file):
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()
        minified = minify_content(content)
        relative = os.path.relpath(file_path, ROOT_DIR)
        out_file.write(f">>> {relative}\n{minified}\n")
    except Exception as err:
        print(f"❌ Error reading file {file_path}: {err}")

def walk_dir(directory, out_file):
    try:
        with os.scandir(directory) as entries:
            for entry in entries:
                if should_exclude(entry.name, entry.path):
                    continue
                if entry.is_dir():
                    walk_dir(entry.path, out_file)
                elif entry.is_file() and is_source_file(entry.name):
                    write_file_to_output(entry.path, out_file)
    except OSError as e:
        print(f"❌ Error accessing directory {directory}: {e}")

def split_file_if_needed(file_path):
    """
    Si el archivo supera MAX_UNIFIED_SIZE, lo divide en trozos de MAX_CHUNK_SIZE.
    Los fragmentos se guardan como .txt para que sean legibles.
    """
    size = os.path.getsize(file_path)
    if size <= MAX_UNIFIED_SIZE:
        print(f"📦 No hace falta dividir el archivo ({size / 1024:.2f} KB).")
        return

    print(f"📏 Archivo mayor a 1 MB ({size / 1024:.2f} KB). Dividiendo…")
    base_name = file_path.replace(".txt", "")          # quitamos .txt temporalmente
    with open(file_path, "r", encoding="utf-8") as src:
        part_num = 1
        while True:
            chunk = src.read(MAX_CHUNK_SIZE)
            if not chunk:
                break
            part_name = f"{base_name}.part{part_num:04d}.txt"
            with open(part_name, "w", encoding="utf-8") as part_file:
                part_file.write(chunk)
            print(f"  ✔ Creado {part_name} ({len(chunk.encode('utf-8')) / 1024:.2f} KB)")
            part_num += 1

    # Borramos el original grande para evitar duplicados
    os.remove(file_path)
    print("🗑 Archivo original eliminado tras la división.")

def consolidate_code():
    output_path = os.path.join(ROOT_DIR, OUTPUT_FILE)
    print(f"📁 Starting minified consolidation from: {ROOT_DIR}")

    if not os.path.exists(ROOT_DIR):
        print(f"❌ Root directory not found: {ROOT_DIR}")
        return

    try:
        with open(output_path, "w", encoding="utf-8") as out_stream:
            walk_dir(ROOT_DIR, out_stream)

        print(f"✅ Code minified and consolidated into: {OUTPUT_FILE}")
        split_file_if_needed(output_path)

    except Exception as e:
        print(f"❌ Failed to create output file: {e}")

# ------------------------------------------------------------------
# RUN
# ------------------------------------------------------------------
if __name__ == "__main__":
    consolidate_code()