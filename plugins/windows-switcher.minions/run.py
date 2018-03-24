#! /usr/bin/env python
# -*- coding: utf-8 -*-

import gi
import os
import json
import tempfile

gi.require_version('Gtk', '3.0')
from gi.repository import Wnck


screen = Wnck.Screen.get_default()
screen.force_update()
tmpdir = tempfile.mkdtemp()

ret = []

for win in screen.get_windows():
    item = {
        'title': win.get_name(),
        'badge': win.get_class_group_name(),
        'action': 'wmctrl -i -a {}'.format(win.get_xid()),
    }

    try:
        ws = win.get_workspace()
        item['subtitle'] = 'Workspace {}'.format(ws.get_name())
    except Exception:
        pass

    try:
        icon = win.get_icon()
        iconpath = os.path.join(tmpdir, '{}.png'.format(win.get_xid()))
        icon.savev(iconpath, 'png', (), ())
        item['icon'] = 'file:{}'.format(iconpath)
    except Exception:
        pass

    ret.append(item)

print(json.dumps(ret, indent=4))
