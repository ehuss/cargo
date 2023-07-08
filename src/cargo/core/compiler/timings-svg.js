const X_LINE = 50;
const MARGIN = 5;
const Y_LINE = 35; // relative to bottom
const PX_PER_SEC = 20.0;
const MIN_TICK_DIST = 50;

function svgNode(name, attrs) {
  let n = document.createElementNS("http://www.w3.org/2000/svg", name);
  for (const attr in attrs) {
    n.setAttributeNS(null, attr, attrs[attr]);
  }
  return n;
}

function render_pipeline_graph() {
  let g = document.getElementById('pipeline-graph');
  const BOX_HEIGHT = 25;
  const Y_TICK_DIST = BOX_HEIGHT + 2;

  let graph_height = Y_TICK_DIST * UNIT_DATA.length;
  let graph_width = draw_graph_axes(g, graph_height);

  // Draw Y tick marks.
  let y_ticks_d = '';
  for (n=1; n<UNIT_DATA.length; n++) {
    let y = graph_height - (n * Y_TICK_DIST);
    y_ticks_d += `M ${X_LINE} ${y} l -5 0 `;
  }
  let y_ticks = svgNode('path', {class: 'graph-axes', d: y_ticks_d});
  g.appendChild(y_ticks);

  // Draw Y labels.
  for (n=0; n<UNIT_DATA.length; n++) {
    let y = MARGIN + (Y_TICK_DIST * (n + 1)) - 13;
    let text = svgNode('text', {x: X_LINE-4, y: y, class: 'graph-label-v'});
    text.textContent = n+1;
    g.appendChild(text);
  }

  // Draw the graph.
  let graph = svgNode("svg", {x: X_LINE, y: MARGIN, width: graph_width, height: graph_height});
  g.appendChild(graph);

  // Compute x,y coordinate of each block.
  let unit_coords = {};
  for (i=0; i<UNIT_DATA.length; i++) {
    let unit = UNIT_DATA[i];
    let y = i * Y_TICK_DIST + 1;
    let x = PX_PER_SEC * unit.start;
    unit_coords[i] = {x, y};
  }
  // Draw the blocks.
  for (i=0; i<UNIT_DATA.length; i++) {
    let unit = UNIT_DATA[i];
    let {x, y} = unit_coords[i];
    let width = Math.max(PX_PER_SEC * unit.duration, 1.0);

    let dep_class = `dep-${i}`;
    let cls = unit.mode == 'run-custom-build' ? 'unit-block-custom' : 'unit-block';
    let rect = svgNode('rect', {x, y, width, height: BOX_HEIGHT, class: cls, 'data-dep-class': dep_class, rx: 3});
    graph.appendChild(rect);

    function draw_dep_lines(x, units) {
      for (unlocked of units) {
        let {x: u_x, y: u_y} = unit_coords[unlocked];
        let dep_d = `M ${x} ${y+BOX_HEIGHT / 2} l -5 0 l 0 ${u_y - y} l ${u_x - x + 5.0} 0`;
        let path = svgNode('path', {class: `${dep_class} dep-line`, d: dep_d});
        graph.appendChild(path);
      }
    }

    let ct = codegen_time(unit);
    if (ct != null) {
      let [rmeta_time, ctime, _cent] = ct;
      let rmeta_x = x + PX_PER_SEC * rmeta_time;
      let rect = svgNode('rect', {x: rmeta_x, y, width: PX_PER_SEC * ctime, height: BOX_HEIGHT, class: 'unit-block-codegen', rx: 3});
      graph.appendChild(rect);
      draw_dep_lines(rmeta_x, unit.unlocked_rmeta_units);
    }
    let label = svgNode('text', {x: x + 5.0, y: y + BOX_HEIGHT / 2 - 5, class: 'unit-label'});
    label.textContent = `${unit.name}${unit.target} ${unit.duration}s`;
    graph.appendChild(label);
    draw_dep_lines(x + width, unit.unlocked_units);
  }
  return graph_width;
}

function render_timing_graph(graph_width) {
  if (graph_width == 0 || CONCURRENCY_DATA.length == 0) {
    return;
  }
  const HEIGHT = 400;
  const AXIS_HEIGHT = HEIGHT - MARGIN - Y_LINE;
  const TOP_MARGIN = 10;
  const GRAPH_HEIGHT = AXIS_HEIGHT - TOP_MARGIN;

  let g = document.getElementById('timing-graph');
  draw_graph_axes(g, AXIS_HEIGHT);

  // Draw Y tick marks and labels.
  let y_ticks_d = '';
  let max_v = 0;
  for (c of CONCURRENCY_DATA) {
    max_v = Math.max(max_v, c.active, c.waiting, c.inactive);
  }
  let [step, top] = split_ticks(max_v, GRAPH_HEIGHT / MIN_TICK_DIST);
  let num_ticks = top / step;
  let tick_dist = GRAPH_HEIGHT / num_ticks;
  for (n=0; n<num_ticks; n++) {
    let y = HEIGHT - Y_LINE - ((n + 1) * tick_dist);
    y_ticks_d += `M ${X_LINE} ${y} l -5 0 `;
    let label = svgNode('text', {x: X_LINE-10, y: y+5, class: 'graph-lavel-v'});
    label.textContent = (n+1) * step;
    g.appendChild(label);
  }
  let y_path = svgNode('path', {class: 'graph-axes', d: y_ticks_d});
  g.appendChild(y_path);

  // Label the Y axis.
  let label_y = (HEIGHT - Y_LINE) / 2;
  let y_label = svgNode('text', {x: 15, y: label_y, class: 'graph-label-v', transform: `rotate(-90, 15, ${label_y})`});
  y_label.textContent = '# Units';
  g.appendChild(y_label);

  // Draw the graph.
  let graph = svgNode('svg', {x: X_LINE, y: MARGIN, width: graph_width, height: GRAPH_HEIGHT + TOP_MARGIN});
  g.appendChild(graph);

  function coord(t, v) {
    return {
      x: graph_width * (t/DURATION),
      y: TOP_MARGIN + GRAPH_HEIGHT * (1.0 - (v / max_v))
    };
  }
  function draw_line(cls, key) {
    let first = CONCURRENCY_DATA[0];
    let last = coord(first.t, key(first));
    let points = '';
    for (c of CONCURRENCY_DATA) {
      let {x, y} = coord(c.t, key(c));
      points += `${x},${last.y} ${x},${y} `;
      last = {x, y};
    }
    let pl = svgNode('polyline', {points, class: cls});
    graph.appendChild(pl);
  }

  draw_line('line-inactive', function(c) {return c.inactive;});
  draw_line('line-waiting', function(c) {return c.waiting;});
  draw_line('line-active', function(c) {return c.active;});

  // Draw a legend.
  let placeholder = document.createElement('div');
  placeholder.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" x="${graph_width-120}"
    width="100" height="62">
    <rect width="100%" height="100%" fill="white" stroke="black" stroke-width="1" />
    <line x1="5" y1="10" x2="50" y2="10" stroke="red" stroke-width="2"/>
    <text x="54" y="11" dominant-baseline="middle" font-size="12px">Waiting</text>

    <line x1="5" y1="50" x2="50" y2="50" stroke="green" stroke-width="2"/>
    <text x="54" y="51" dominant-baseline="middle" font-size="12px">Active</text>

    <line x1="5" y1="30" x2="50" y2="30" stroke="blue" stroke-width="2"/>
    <text x="54" y="31" dominant-baseline="middle" font-size="12px">Inactive</text>
    </svg>`;
  graph.appendChild(placeholder.firstChild);
}

function draw_graph_axes(g, graph_height) {
  let graph_width = PX_PER_SEC * DURATION;
  let width = graph_width + X_LINE + 30;
  let height = graph_height + MARGIN + Y_LINE;
  g.setAttribute("width", width);
  g.setAttribute("height", height);

  let labels = [];
  let graph_axes_d = `M ${X_LINE} ${MARGIN}
    l 0 ${graph_height}
    l ${graph_width+20} 0 `;

  // Draw X tick marks.
  let tick_width = graph_width - 10;
  let [step, top] = split_ticks(DURATION, tick_width / MIN_TICK_DIST);
  let num_ticks = top / step;
  let tick_dist = tick_width / num_ticks;
  for (n=0; n<num_ticks; n++) {
      let x = X_LINE + ((n + 1) * tick_dist);
      graph_axes_d += `M ${x} ${height-Y_LINE} l 0 5 `;
      let label = svgNode('text', {x: x, y: height - Y_LINE + 20, class: 'graph-label-h'});
      label.textContent = `${(n+1) * step}s`;
      g.appendChild(label);
  }
  let graph_axes = svgNode('path', {class: 'graph-axes', d: graph_axes_d});
  g.appendChild(graph_axes);

  // Draw vertical lines.
  let vert_d = '';
  for (n=0; n<num_ticks; n++) {
    let x = X_LINE + ((n + 1) * tick_dist);
    vert_d += `M ${x} ${MARGIN} l 0 ${graph_height} `;
  }
  let vert_line = svgNode('path', {class: 'vert-line', d: vert_d});
  g.appendChild(vert_line);
  return graph_width;
}

function round_up(n, step) {
  if (n % step == 0) {
    return n;
  } else {
    return (step - n % step) + n;
  }
}

// Determine the `(step, max_value)` of the number of ticks along an axis.
function split_ticks(n, max_ticks) {
  if (n <= max_ticks) {
    return [1, n];
  } else if (n <= max_ticks * 2) {
    return [2, round_up(n, 2)];
  } else if (n <= max_ticks * 4) {
    return [4, round_up(n, 4)];
  } else if (n <= max_ticks * 5) {
    return [5, round_up(n, 5)];
  } else {
    let step = 10;
    while (true) {
      let top = round_up(n, step);
      if (top <= max_ticks * step) {
        return [step, top];
      }
      step += 10;
    }
  }
}

function codegen_time(unit) {
  if (unit.rmeta_time == null) {
    return null;
  }
  let ctime = unit.duration - unit.rmeta_time;
  let cent = (ctime / unit.duration) * 100.0;
  return [unit.rmeta_time, ctime, cent];
}

function show_deps(event) {
  for (const el of document.getElementsByClassName('dep-line')) {
    el.classList.remove('dep-line-highlight');
  }
  for (const el of document.getElementsByClassName(event.currentTarget.dataset.depClass)) {
    el.classList.add('dep-line-highlight');
  }
}

let graph_width = render_pipeline_graph();
render_timing_graph(graph_width);
for (const el of document.getElementsByClassName('unit-block')) {
  el.onmouseover = show_deps;
}
