#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re
import json
from datetime import datetime


EMOJIPEDIA_URL = 'https://emojipedia.org/emoji/'
EMOJI_DATA_PATH = os.path.expanduser('~/.minions/emoji.html')
EMOJI_PATTERN = (r'<a\s+href="/emoji/[^/]*/">' +
                 r'\s*<span\s+class="emoji">' +
                 r'\s*([^<])' +  # ignore multi-char emoji for now
                 r'\s*</span>' +
                 r'\s*([^<]+)' +
                 r'</a>')


data_last_update = 'Never'
data = []

try:
    content = open(EMOJI_DATA_PATH).read()
    data_last_update = os.stat(EMOJI_DATA_PATH).st_mtime
except OSError:
    pass
else:
    data_last_update = datetime.fromtimestamp(data_last_update).ctime()
    for m in re.finditer(EMOJI_PATTERN, content):
        emoji = m.group(1).strip()
        desc = m.group(2).strip()
        data.append({
            'title': desc,
            'icon': 'character::{}'.format(emoji),
            'data': emoji,
        })

data.append({
    'title': 'Update Emoji data from emojipedia.org',
    'subtitle': 'Last update: {}'.format(data_last_update),
    'priority': 100,
    'action': ('bash -c "wget {} -O {}"'
               .format(EMOJIPEDIA_URL, EMOJI_DATA_PATH)),
})

print(json.dumps(data, indent=4))
