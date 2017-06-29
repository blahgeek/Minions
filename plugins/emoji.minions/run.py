#!/usr/bin/env python
# -*- coding: utf-8 -*-
# @Author: BlahGeek
# @Date:   2017-06-18
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-06-29

from __future__ import print_function
from os import path
import json


data = json.load(open(path.join(path.dirname(__file__), 'emojis.json')))
data = [{
            'title': name,
            'icon': 'character::{}'.format(props['char']),
            'subtitle': ' '.join(props['keywords']),
            'badge': props['category'],
            'data_text': props['char'],
        } for name, props in data.items() if props.get('char')]

print(json.dumps({'results': data}, indent=4))
