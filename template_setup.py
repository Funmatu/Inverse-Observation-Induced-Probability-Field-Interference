# template_setup.py
import os
import sys


def replace_in_file(filepath, old_kebab, new_kebab, old_snake, new_snake):
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()

        new_content = content.replace(old_kebab, new_kebab).replace(
            old_snake, new_snake
        )

        if content != new_content:
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(new_content)
            print(f"Updated: {filepath}")
    except FileNotFoundError:
        print(f"Skipped (not found): {filepath}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python template_setup.py <new-project-name>")
        print("Example: python template_setup.py my-robot-sim")
        return

    new_name_kebab = sys.argv[1]  # 例: my-robot-sim
    new_name_snake = new_name_kebab.replace("-", "_")  # 例: my_robot_sim

    # 現在のプロジェクト名（テンプレートの元名）
    OLD_NAME_KEBAB = "nx-compute-rs"
    OLD_NAME_SNAKE = "nx_compute_rs"

    # 書き換え対象のファイルリスト
    target_files = [
        "Cargo.toml",
        "pyproject.toml",
        "www/index.js",
        "www/index.html",
        ".github/workflows/deploy.yml",
    ]

    print(f"Renaming project from '{OLD_NAME_KEBAB}' to '{new_name_kebab}'...")

    for file in target_files:
        replace_in_file(
            file, OLD_NAME_KEBAB, new_name_kebab, OLD_NAME_SNAKE, new_name_snake
        )

    print("\nDone! Don't forget to run:")
    print(f"  rm template_setup.py")
    print(f"  git commit -am 'setup: initialize project {new_name_kebab}'")


if __name__ == "__main__":
    main()
