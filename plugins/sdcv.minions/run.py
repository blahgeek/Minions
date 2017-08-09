#!/usr/bin/env python
# -*- coding: utf-8 -*-
# @Author: BlahGeek
# @Date:   2017-08-06
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-08-09

import os
import sys
import json
import subprocess


def lookup(word, fuzzy=False):
    if fuzzy:
        word = '/' + word
    output = subprocess.check_output(['sdcv', '--utf8-output', '--utf8-input',
                                      '-n', word],
                                     universal_newlines=True)
    result = []
    item = []
    for line in output.splitlines():
        if not line:
            continue
        if line.startswith('-->'):
            if len(item) > 2:
                result.append(item)
                item = []
            item.append(line[3:])
        else:
            if item:
                item.append(line)

    if len(item) > 2:
        result.append(item)

    return result


if __name__ == '__main__':
    word = sys.argv[1]
    arg_type = os.getenv('MINIONS_ARG_TYPE', 'text')
    res = lookup(word, arg_type == 'text_realtime')

    res = [{
        'title': item[1],
        'subtitle': '; '.join(item[2:]),
        'badge': item[0],
        'icon': 'file:stardict.png',
    } for item in res]

    print(json.dumps(res, indent=4))
