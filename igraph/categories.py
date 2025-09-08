import math
import pickle
import sqlite3
import time

import igraph as ig
from tqdm import tqdm

from main import clamp

con = sqlite3.connect("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240401/dewiki_cl_database.sqlite")
con_title_id = sqlite3.connect(
    "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240401/dewiki_title_id_all_database.sqlite")


def id_titlemap():
    id_title_map = {}
    for (page_id, page_title) in con_title_id.execute("""SELECT page_id, page_title FROM WikiPage"""):
        id_title_map[page_id] = page_title
    return id_title_map


rows = con.execute(f"""SELECT page_id_from, category_name
FROM WikiCategoryLinks where category_type == 'subcat'""")

# rows = con.execute(f"""SELECT page_id_from, category_name
# FROM WikiCategoryLinks WHERE page_id_from in
#  (select page_id_from from WikiCategoryLinks where category_type == 'subcat' limit 10000) """)

g = ig.Graph(directed=True)
limit = 1_388_365
pname_vid = {}
edges = []
id_title_map = id_titlemap()
print("loaded map")

subcat_count_map = {}
subcat_count_hist = {}

rows = rows.fetchall()
for (page_id, target_name) in tqdm(rows, total=limit):
    if target_name not in subcat_count_map:
        subcat_count_map[target_name] = 0

    # if page_id not in neighbors_count_map:
    #     neighbors_count_map[page_id] = 0
    #
    subcat_count_map[target_name] += 1

for v in subcat_count_map.values():
    if v not in subcat_count_hist:
        subcat_count_hist[v] = 0
    subcat_count_hist[v] += 1

print(subcat_count_hist)
min_size = 15
max_size = 150

def node_size(value):
    return clamp(math.sqrt(value * 5), min_size, max_size)


MIN_SUBCATS = 1
for (page_id, target_name) in tqdm(rows, total=limit):
    source_name = id_title_map[page_id]
    if subcat_count_map[target_name] <= MIN_SUBCATS or subcat_count_map.get(source_name, 0) <= MIN_SUBCATS:
        continue

    # print(source_name)
    # print(target_name)
    # print(subcat_count_map[target_name])

    if source_name not in pname_vid:
        vid = len(pname_vid)
        pname_vid[source_name] = vid
        g.add_vertex(name=source_name, size=node_size(subcat_count_map[source_name]))

    if target_name not in pname_vid:
        vid = len(pname_vid)
        pname_vid[target_name] = vid
        g.add_vertex(name=target_name, size=node_size(subcat_count_map[target_name]))

    if source_name != target_name:
        edges.append((pname_vid[source_name], pname_vid[target_name]))

# g.add_vertices(len(pname_vid))
g.add_edges(edges)

# print(neighbors_hist)
print("nodes: ", len(g.vs))

# t1 = time.time()
# layout = g.layout_kamada_kawai()
# print("made layout: " + str(time.time() - t1) + "s")
# pickle.dump(layout, open("categories_kk.layout", "wb"))

layout = pickle.load(open("categories_kk.layout", "rb"))
print("loaded layout file")

scale = 10

visual_style = {}

# visual_style["vertex_size"] = [clamp(len(v.neighbors()), 7, 75) for v in g.vs]
visual_style["vertex_size"] = g.vs["size"]
# visual_style["vertex_color"] = [color_dict[gender] for gender in g.vs["gender"]]
# visual_style["vertex_label"] = g.vs["name"]
visual_style["vertex_label"] = g.vs["name"]
visual_style["vertex_label_size"] = [clamp(math.sqrt(s) * 2, 12, 75) if s >= 40 else 0 for s in g.vs["size"]]
# visual_style["vertex_label_dist"] = 0

visual_style["edge_width"] = 1
visual_style["edge_color"] = "#D3D3D3"

visual_style["layout"] = layout
visual_style["bbox"] = (1920 * scale, 1080 * scale)
visual_style["hovermode"] = "closest"

ig.plot(g, "categories2.png", **visual_style)
