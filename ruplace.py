import argparse
import re
from pathlib import Path

import cli_ui as ui
from inflection import camelize, dasherize, underscore


def screamize(x):
    return camelize(x).upper()


def ruplace(kind, path, lineno, input, pattern, replacement):
    if kind == "substring":
        input_fragments, output_fragments = get_fragments_substring(
            input, pattern, replacement
        )
    elif kind == "subvert":
        input_fragments, output_fragments = get_fragments_subvert(
            input, pattern, replacement
        )
    elif kind == "regex":
        regex = re.compile(pattern)
        input_fragments, output_fragments = get_fragments_regex(
            input, regex, replacement
        )
    if not input_fragments:
        return None
    output = get_output(input, input_fragments, output_fragments)
    print_red(path, lineno, input, input_fragments)
    print_green(path, lineno, output, output_fragments)
    return output


def get_fragments_substring(input, pattern, replacement):
    input_fragments = []
    output_fragments = []

    input_index = 0
    output_index = 0
    buff = input
    while True:
        pos = buff.find(pattern)
        if pos == -1:
            break
        input_index += pos
        output_index += pos
        input_fragments.append((input_index, pattern))
        output_fragments.append((output_index, replacement))
        buff = buff[input_index + len(pattern) :]
        input_index += len(pattern)
        output_index += len(replacement)

    return (input_fragments, output_fragments)


def subvert(buff, patterns, replacements):
    candidates = []
    best_pos = len(buff)
    best_pattern_index = None
    for i, pattern in enumerate(patterns):
        pos = buff.find(pattern)
        if pos != -1 and pos < best_pos:
            best_pos = pos
            best_pattern_index = i
    if best_pattern_index is None:
        return -1, None, None
    else:
        return best_pos, patterns[best_pattern_index], replacements[best_pattern_index]


def get_fragments_subvert(input, pattern, replacement):
    input_fragments = []
    output_fragments = []

    patterns = []
    replacements = []
    funcs = [camelize, underscore, dasherize, screamize]
    for func in funcs:
        patterns.append(func(pattern))
        replacements.append(func(replacement))

    input_index = 0
    output_index = 0
    buff = input
    while True:
        pos, pattern, replacement = subvert(buff, patterns, replacements)
        if pos == -1:
            break
        input_index += pos
        output_index += pos
        input_fragments.append((input_index, pattern))
        output_fragments.append((output_index, replacement))
        buff = buff[input_index + len(pattern) :]
        input_index += len(pattern)
        output_index += len(replacement)

    return (input_fragments, output_fragments)


def get_fragments_regex(input, regex, replacement):
    input_fragments = []
    output_fragments = []

    input_index = 0
    output_index = 0
    buff = input
    while True:
        match = regex.search(buff)
        if match is None:
            break
        group = match.group()
        input_index += match.start()
        output_index += match.start()
        input_fragments.append((input_index, group))
        new_string = regex.sub(replacement, group)
        output_fragments.append((output_index, new_string))
        buff = buff[input_index + len(group) :]
        input_index += len(group)
        output_index += len(new_string)

    return (input_fragments, output_fragments)


def get_output(input, input_fragments, output_fragments):
    current_index = 0
    output = ""
    for (in_index, in_substring), (out_index, out_substring) in zip(
        input_fragments, output_fragments
    ):
        output += input[current_index:in_index]
        output += out_substring
        current_index = in_index + len(in_substring)
    output += input[current_index:]
    return output


def print_red(path, lineno, input, input_fragments):
    ui.info(ui.bold, f"{path}:{lineno}", ui.red, "--- ", end="")
    current_index = 0
    for (in_index, in_substring) in input_fragments:
        ui.info(input[current_index:in_index], end="")
        ui.info(ui.red, ui.underline, in_substring, end="")
        current_index = in_index + len(in_substring)
    ui.info(input[current_index:])


def print_green(path, lineno, output, output_fragments):
    ui.info(ui.bold, f"{path}:{lineno}", ui.green, "+++ ", end="")
    current_index = 0
    for (out_index, out_substring) in output_fragments:
        ui.info(output[current_index:out_index], end="")
        ui.info(ui.green, ui.underline, out_substring, end="")
        current_index = out_index + len(out_substring)
    ui.info(output[current_index:])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("src_path", type=Path)
    parser.add_argument("pattern")
    parser.add_argument("replacement")
    parser.add_argument("--regex", action="store_true")
    parser.add_argument("--subvert", action="store_true")

    args = parser.parse_args()
    contents = args.src_path.read_text()
    kind = "substring"
    if args.subvert:
        kind = "subvert"
    if args.regex:
        kind = "regex"
    for (i, line) in enumerate(contents.splitlines()):
        res = ruplace(kind, args.src_path, i + 1, line, args.pattern, args.replacement)
        if res:
            print()


if __name__ == "__main__":
    main()
