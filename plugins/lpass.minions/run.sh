#!/bin/bash -ex

export LPASS_AUTO_SYNC_TIME=3600

echo '['

lpass ls --format '{
_QUOTE_title_QUOTE_: _QUOTE_%an_QUOTE_,
_QUOTE_subtitle_QUOTE_: _QUOTE_%au_QUOTE_,
_QUOTE_badge_QUOTE_: _QUOTE_%ag_QUOTE_,
_QUOTE_data_text_QUOTE_: _QUOTE_%ap_QUOTE_,
_QUOTE_icon_QUOTE_: _QUOTE_character::_QUOTE_
},' | sed 's/\\/\\\\/g; s/"/\\"/g; s/_QUOTE_/"/g; $ s/,$//'

echo ']'
