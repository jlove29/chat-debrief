import re
import sys
import argparse
import os

def convert_to_bookmarklet(js_code):
    # 1. Remove single-line comments (// ...)
    # This regex looks for //, but ensures it's not inside a string or URL (http://)
    # Note: This is a basic stripper. For complex code, a real minifier is safer.
    js_code = re.sub(r'^\s*//.*$', '', js_code, flags=re.MULTILINE) # Remove full line comments
    js_code = re.sub(r'(?<!:)\/\/.*$', '', js_code, flags=re.MULTILINE) # Remove inline comments

    # 2. Remove multi-line comments (/* ... */)
    js_code = re.sub(r'/\*[\s\S]*?\*/', '', js_code)

    # 3. Replace newlines with spaces to create a single line
    js_code = js_code.replace('\n', ' ').replace('\r', '')

    # 4. Collapse multiple spaces into one to save space
    js_code = re.sub(r'\s+', ' ', js_code)

    # 5. Escape single quotes slightly (optional, but helps prevents breakage)
    # js_code = js_code.replace("'", "%27") 

    # 6. Wrap in IIFE (Immediately Invoked Function Expression)
    # This ensures variables don't leak and it executes immediately.
    bookmarklet = f"javascript:(function(){{ {js_code.strip()} }})();"

    return bookmarklet

def main():
    parser = argparse.ArgumentParser(description='Convert JavaScript to bookmarklet')
    parser.add_argument('type', choices=['gemini', 'aim', 'chat'], 
                        help='Type of bookmarklet: gemini, aim, or chat')
    args = parser.parse_args()
    
    # Construct the filename based on the type argument
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    print(f"--- Creating {args.type} bookmarklet from {args.type}.js ---")
    
    # Read from the file(s)
    try:
        if args.type == 'chat':
            # For chat, we need to combine gemini.js and aim.js functions with chat.js
            gemini_file = os.path.join(script_dir, "gemini.js")
            aim_file = os.path.join(script_dir, "aim.js")
            chat_file = os.path.join(script_dir, "chat.js")
            
            with open(gemini_file, 'r') as f:
                gemini_code = f.read()
            with open(aim_file, 'r') as f:
                aim_code = f.read()
            with open(chat_file, 'r') as f:
                chat_code = f.read()
            
            # Remove the downloadGemini() call from gemini.js and downloadAim() call from aim.js
            gemini_code = gemini_code.replace('downloadGemini();', '').strip()
            aim_code = aim_code.replace('downloadAim();', '').strip()
            
            # Combine all the code
            input_lines = f"{gemini_code}\n\n{aim_code}\n\n{chat_code}"
        else:
            input_file = os.path.join(script_dir, f"{args.type}.js")
            with open(input_file, 'r') as f:
                input_lines = f.read()
    except FileNotFoundError as e:
        print(f"Error: Required file not found in {script_dir}: {e}")
        return
    except Exception as e:
        print(f"Error reading file: {e}")
        return

    if not input_lines.strip():
        print("No code provided in file.")
        return

    result = convert_to_bookmarklet(input_lines)

    print(f"\n--- COPY THIS {args.type.upper()} BOOKMARKLET BELOW ---\n")
    print(result)
    print("\n------------------------------")

if __name__ == "__main__":
    main()