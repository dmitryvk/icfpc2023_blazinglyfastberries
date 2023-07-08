#!/usr/bin/env python3

import drawsvg as draw
import json
import sys

if len(sys.argv) < 3 or len(sys.argv) > 4:
    print("Usage: vis_prob.py <out.svg> <problem.json> [<sol.json>]")
    exit(-1)

out_name = sys.argv[1]

prob_name = sys.argv[2]
prob = json.load(open(prob_name, "r"))

d = draw.Drawing(prob['room_width'], prob['room_height'])
# room
d.append(draw.Rectangle(0, 0, prob['room_width'], prob['room_height'],
                        fill='none', stroke='black', stroke_opacity=0.5))
# stage
d.append(draw.Rectangle(prob['stage_bottom_left'][0], prob['stage_bottom_left'][1],
                        prob['stage_width'], prob['stage_height'],
                        fill='none', stroke='red'))

# pillars
for pillar in prob['pillars']:
    d.append(draw.Circle(pillar['center'][0], pillar['center'][1],
                         pillar['radius'],
                         color='black', fill='blue'))
# origin
# d.append(draw.Circle(0, 0, 50, fill='orange'))

for id, attendee in enumerate(prob['attendees']):
    x = attendee['x']
    y = attendee['y']
    d.append(draw.Circle(x, y, 1, fill='blue'))
    d.append(draw.Text(str(id), x=x, y=y,
                       font_size=12,
                       text_anchor='middle', dominant_baseline='middle'))

if len(sys.argv) == 4:
    sol_name = sys.argv[3]
    sol = json.load(open(sol_name, "r"))
    for id, musician in enumerate(sol['placements']):
        x = musician['x']
        y = musician['y']
        d.append(draw.Circle(x, y, 10,
                             fill='none', stroke='green'))
        d.append(draw.Circle(x, y, 5,
                             fill='red'))
        d.append(draw.Text(str(id) + '-' + str(prob['musicians'][id]), x=x, y=y,
                           font_size=12,
                           text_anchor='middle', dominant_baseline='middle'))

d.save_svg(out_name)
