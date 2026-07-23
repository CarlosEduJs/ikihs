#!/usr/bin/env python3
"""Convert TextMate .tmLanguage plist to Sublime .sublime-syntax YAML."""

import json
import plistlib
import re
import sys
from collections import OrderedDict

INDENT = '  '

def escape_yaml_string(s):
    if s is None:
        return ''
    s = str(s)
    if not s:
        return "''"
    if '\n' in s or len(s) > 80:
        return json.dumps(s)
    return s

def quote_yaml_string(s):
    if s is None:
        return "''"
    s = str(s)
    if not s:
        return "''"
    return json.dumps(s)

quote_regex = quote_yaml_string

def convert_pattern(p, indent=0, context_name=''):
    entries = []
    if 'include' in p:
        inc = p['include']
        if inc.startswith('#'):
            inc = inc[1:]
        entries.append(f'- include: {inc}')
        return '\n'.join(entries)

    has_match = 'match' in p
    has_begin = 'begin' in p
    has_end = 'end' in p

    prefix = INDENT * indent

    if has_begin and has_end:
        begin_re = p['begin']
        end_re = p['end']
        begin_caps = p.get('beginCaptures', p.get('captures', {}))
        end_caps = p.get('endCaptures', {})
        content_name = p.get('contentName', '')

        entries.append(f'- match: {quote_yaml_string(begin_re)}')
        if begin_caps:
            entries.append(f'  captures:')
            for idx in sorted(begin_caps.keys(), key=int):
                cap = begin_caps[idx]
                sc = cap.get('name', '')
                entries.append(f'    {idx}:')
                entries.append(f'      scope: {quote_yaml_string(sc)}')
        if p.get('name'):
            entries.append(f'  scope: {quote_yaml_string(p["name"])}')
        entries.append(f'  push:')
        entries.append(f'  - match: {quote_yaml_string(end_re)}')
        if end_caps:
            entries.append(f'    captures:')
            for idx in sorted(end_caps.keys(), key=int):
                cap = end_caps[idx]
                sc = cap.get('name', '')
                entries.append(f'      {idx}:')
                entries.append(f'        scope: {quote_yaml_string(sc)}')
        entries.append(f'    pop: true')

        sub_patterns = p.get('patterns', [])
        if content_name:
            entries.append(f'    scope: {quote_yaml_string(content_name)}')
        for sp in sub_patterns:
            entries.append(convert_pattern(sp, indent + 1, context_name))

        return '\n'.join(entries)

    elif has_match:
        match_re = p['match']
        scope = p.get('name', '')
        captures = p.get('captures', {})
        sub_patterns = p.get('patterns', [])

        entries.append(f'- match: {quote_yaml_string(match_re)}')
        if captures:
            entries.append(f'  captures:')
            for idx in sorted(captures.keys(), key=int):
                cap = captures[idx]
                entries.append(f'    {idx}:')
                if 'name' in cap:
                    entries.append(f'      scope: {quote_yaml_string(cap["name"])}')
                if 'patterns' in cap:
                    for sp in cap['patterns']:
                        entries.append(convert_pattern(sp, indent + 3, context_name))
        if scope:
            entries.append(f'  scope: {quote_yaml_string(scope)}')

        if sub_patterns:
            entries.append(f'  push:')
            for sp in sub_patterns:
                entries.append(convert_pattern(sp, indent + 1, context_name))
            entries.append(f'{INDENT * (indent+1)}- match: \'(?!) \'')
            entries.append(f'{INDENT * (indent+2)}pop: true')

        return '\n'.join(entries)

    return ''

def convert_repository(repo):
    """Convert the repository into contexts."""
    contexts = OrderedDict()
    for name, entry in repo.items():
        if isinstance(entry, dict):
            if 'begin' in entry and 'end' in entry:
                patterns = entry.get('patterns', [])
                result = convert_pattern_as_context(entry, name, patterns)
                contexts[name] = result
            elif 'match' in entry:
                result = [convert_pattern(entry, 0, name)]
                contexts[name] = result
            elif 'include' in entry:
                inc = entry['include']
                if inc.startswith('#'):
                    inc = inc[1:]
                contexts[name] = [f'- include: {inc}']
            elif 'patterns' in entry:
                pattern_list = entry['patterns']
                result = []
                for p in pattern_list:
                    result.append(convert_pattern(p, 0, name))
                contexts[name] = result
            else:
                contexts[name] = []
        elif isinstance(entry, list):
            result = []
            for p in entry:
                result.append(convert_pattern(p, 0, name))
            contexts[name] = result
    return contexts

def convert_pattern_as_context(entry, name, sub_patterns):
    """Convert a repository entry with begin/end into a context with push/pop."""
    begin_re = entry.get('begin', '')
    end_re = entry.get('end', '')
    begin_caps = entry.get('beginCaptures', entry.get('captures', {}))
    end_caps = entry.get('endCaptures', {})
    content_name = entry.get('contentName', '')
    entry_name = entry.get('name', '')

    lines = []
    lines.append(f'- match: {quote_yaml_string(begin_re)}')
    if begin_caps:
        lines.append(f'  captures:')
        for idx in sorted(begin_caps.keys(), key=int):
            cap = begin_caps[idx]
            sc = cap.get('name', '')
            lines.append(f'    {idx}:')
            lines.append(f'      scope: {quote_yaml_string(sc)}')
    if entry_name:
        lines.append(f'  scope: {quote_yaml_string(entry_name)}')
    lines.append(f'  push:')

    for sp in sub_patterns:
        sp_text = convert_pattern(sp, 0, name)
        for sp_line in sp_text.split('\n'):
            lines.append(f'    {sp_line}')

    lines.append(f'    - match: {quote_yaml_string(end_re)}')
    if end_caps:
        lines.append(f'      captures:')
        for idx in sorted(end_caps.keys(), key=int):
            cap = end_caps[idx]
            sc = cap.get('name', '')
            lines.append(f'        {idx}:')
            lines.append(f'          scope: {quote_yaml_string(sc)}')
    lines.append(f'      pop: true')
    if content_name:
        lines.append(f'      scope: {quote_yaml_string(content_name)}')

    return [line if isinstance(line, str) else line for line in lines]

def main():
    import json

    input_path = sys.argv[1]
    output_path = sys.argv[2] if len(sys.argv) > 2 else input_path.replace('.tmLanguage', '.sublime-syntax')

    with open(input_path, 'rb') as f:
        tm = plistlib.load(f)

    name = tm.get('name', 'TypeScript')
    scope = tm.get('scopeName', 'source.ts')
    file_extensions = tm.get('fileTypes', ['ts'])
    uuid = tm.get('uuid', '')
    patterns = tm.get('patterns', [])
    repo = tm.get('repository', {})

    lines = ['%YAML 1.2', '---', f'name: {name}', f'scope: {scope}']
    if file_extensions:
        lines.append('file_extensions:')
        for ext in file_extensions:
            lines.append(f'  - {ext}')
    lines.append('')
    lines.append('contexts:')

    # Main context from top-level patterns
    main_patterns = []
    for p in patterns:
        main_patterns.append(convert_pattern(p, 0, 'main'))

    lines.append('  main:')
    for mp in main_patterns:
        for line in mp.split('\n'):
            lines.append(f'    {line}' if line else '')

    # Repository contexts
    for ctx_name, ctx_patterns in convert_repository(repo).items():
        lines.append(f'  {ctx_name}:')
        for cp in ctx_patterns:
            for line in cp.split('\n'):
                if line:
                    lines.append(f'    {line}')

    with open(output_path, 'w') as f:
        f.write('\n'.join(lines))
        f.write('\n')

    print(f"Converted {input_path} -> {output_path}")
    print(f"  Patterns: {len(patterns)} top-level, {len(repo)} repo contexts")

if __name__ == '__main__':
    main()
