import re
import sys


def process_message(message):
    if isinstance(message, bytes):
        message = message.decode("utf-8", errors="replace")

    lines = message.splitlines()
    if not lines or not lines[0].strip():
        return "no commit message"

    first_line = lines[0]
    match = re.match(r"^(\w+): (.+)", first_line)
    if match:
        scope = match.group(1)
        msg = match.group(2)
        msg = re.sub(r"\badd\b", "", msg, flags=re.IGNORECASE)
        msg = re.sub(r"\s+", " ", msg).strip()
        new_msg = f"[{scope}] {msg}"[:60]
        return new_msg
    else:
        return first_line[:60]


if __name__ == "__main__":
    print(process_message(sys.stdin.read()))
