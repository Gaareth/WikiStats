import datetime
import os
import sqlite3
import time

import colorcet as cc
import dateutil.relativedelta
import matplotlib
import matplotlib.pyplot as plt
import requests
from tqdm import tqdm

import igraph as ig

from dotenv import load_dotenv

load_dotenv()  # take environment variables from .env.
t1 = time.time()


def clamp(value, min_value, max_value):
    return min(max(min_value, value), max_value)


def id_titlemap(con):
    id_title_map = {}
    for (page_id, page_title) in con.execute("""SELECT page_id, page_title FROM WikiPage"""):
        id_title_map[page_id] = page_title
    return id_title_map


def title_to_pid(con, title):
    res = con.execute("""SELECT page_id FROM WikiPage where page_title = ?""", (title,)).fetchone()
    if res is None:
        return None
    return res[0]


def to_edgelist(con, limit=10):
    nodes = set()
    limit_str = f"where page_id in (select page_id from WikiLink limit {limit})"
    with open("edges.csv", "w") as f:
        f.write(f"source, target\n")
        for (page_id, page_link) in con.execute(f"SELECT page_id, page_link FROM WikiLink {limit_str}"):
            f.write(f"{page_id}, {page_link}\n")
            # nodes.add(page_id)
            # nodes.add(page_link)
    return nodes


def to_nodelist(list=[]):
    # id_title_map = id_titlemap()
    print("loaded idtitle map")
    with open("nodes.csv", "w") as f:
        f.write(f"id, label\n")
        for (pid, title) in con.execute("""SELECT page_id, page_title FROM WikiPage"""):
            f.write(f"{pid}, {title}\n")


def int2rgb(n):
    b = n % 256
    g = int(((n - b) / 256) % 256)
    r = int(((n - b) / 256 ** 2) - g / 256)
    return r, g, b


def rgb2hex(rgb) -> str:
    (r, g, b) = rgb
    print(rgb)
    return '#%02x%02x%02x' % (r, g, b)


def get_fg(bg_rgb: tuple):
    (red, green, blue) = bg_rgb
    if (red * 0.299 + green * 0.587 + blue * 0.114) > 186:
        return "#000000"
    return "#ffffff"


def most_popular_pages():
    project = "de.wikipedia"
    dt = datetime.datetime.now() + dateutil.relativedelta.relativedelta(months=-1)
    month = f"0{dt.month}" if dt.month else dt.month

    url = f"https://wikimedia.org/api/rest_v1/metrics/pageviews/top/{project}/all-access/{dt.year}/{month}/all-days"
    r = requests.get(url, headers={
        "user-agent": os.environ["WIKIPEDIA_REST_API_USER_AGENT"]
    })

    if not r.ok:
        print(r.json())
        quit(-1)

    articles = r.json()["items"][0]["articles"]
    return articles


def fetch_rows_limit(con, limit):
    return con.execute(f"""SELECT page_id, page_link FROM WikiLink
                    where page_id in (select page_id from WikiLink limit {limit})""").fetchall()


def fetch_all_rows(con):
    return con.execute("""SELECT page_id, page_link FROM WikiLink""").fetchall()


def load_comm_map_category():
    pid_comm_map = {}
    comms = set()
    # category_communities
    with open("category_communities_max_ref.csv", "r") as f:
        for i, row in enumerate(f.readlines()):
            if i == 0:
                continue

            comm_id = row.split(",")[0].strip()
            pid_id = row.split(",")[1].strip()
            pid_comm_map[pid_id] = comm_id.__hash__() % len(cc.glasbey_bw)
            # pid_comm_map[int(pid_id)] = int(comm_id)
            comms.add(comm_id)
    print("comms: ", len(comms))
    # print(pid_comm_map[1])
    return pid_comm_map


def load_comm_map_communities(file_path):
    pid_comm_map = {}
    comms = set()
    with open(file_path, "r") as f:
        for i, row in enumerate(f.readlines()):
            if i == 0:
                continue

            comm_id = row.split(",")[0].strip()
            pid_id = row.split(",")[1].strip()
            pid_comm_map[int(pid_id)] = int(comm_id)
            comms.add(comm_id)

    print("comms: ", len(comms))
    return pid_comm_map


def create_graph(rows, con, pid_comm_map):
    g = ig.Graph(directed=False)
    pid_vid = {}
    edges = []

    # 1_000_000 ~= 15s la package
    # ~ 3s igraph package
    # 5M: 33s
    # 50M: 400s
    # 100M: ?

    vertex_labels = []

    id_title_map = id_titlemap(con)

    def num_links(pid):
        c = 0
        for (r_pid, _) in rows:
            if pid == r_pid:
                c += 1
        return c

    MAX_VERTEX_SIZE = 100

    def calc_size(l):
        return clamp(l, 10, MAX_VERTEX_SIZE)

    def build_num_links_map(rows):
        """ Returns map mapping page_id to times used in rows"""
        num_links_map = {}
        for (page_id, page_link) in tqdm(rows, total=len(rows)):
            # if page_id not in links_map:
            #     links_map[page_id] = []
            # links_map[page_id].append(page_link)

            if page_id not in num_links_map:
                num_links_map[page_id] = 0
            num_links_map[page_id] += 1

            if page_link not in num_links_map:
                num_links_map[page_link] = 0
            num_links_map[page_link] += 1

        return num_links_map

    num_links_map = build_num_links_map(rows)

    comm_lens = {}
    MAX_COMM_NODES = 100

    pal = cc.glasbey_bw

    def add_vertex(pid):
        if pid not in pid_vid:
            l = num_links_map[pid]
            size = calc_size(l)
            name = id_title_map[pid]
            # name = id_title_map[pid] if size >= MAX_VERTEX_SIZE else ""

            comm = 0
            # if pid in pid_comm_map:
            #     comm = pid_comm_map[pid]

            # if name not in pid_comm_map:
            #     return None
            #
            # comm = pid_comm_map[name]

            # color = matplotlib.colors.to_hex(pal[comm])
            # print(color)

            if comm not in comm_lens:
                comm_lens[comm] = 0

            if comm_lens[comm] < MAX_COMM_NODES or True:
                comm_lens[comm] += 1
                # g.add_vertex(name=name if l > 1000 else "", size=calc_size(l), comm=pid_comm_map[page_id])
                # vertex_labels[page_id] = id_title_map[page_id]
                g.add_vertex(label=name, size=size)
                vid = len(pid_vid)
                pid_vid[pid] = vid
                # vid_pid[vid] = page_id
            else:
                return None
        return 1

    for (page_id, page_link) in tqdm(rows, total=len(rows)):
        # print(page_id, page_link)
        # vertices.add(page_id)
        # vertices.add(page_link)

        ret = add_vertex(page_id)
        if ret is None:
            continue
        ret = add_vertex(page_link)
        if ret is None:
            continue

        # comm = pid_comm_map[page_id]
        #
        # color = matplotlib.colors.to_hex(pal[comm])
        edges.append((pid_vid[page_id], pid_vid[page_link]))
        # g.add_edge(source=pid_vid[page_id], target=pid_vid[page_link], color=color)

    print("made edges")
    print("vertices:", len(pid_vid))
    # g.add_vertices(len(pid_vid))
    # # g.vs["name"] = [str(v) for v in pid_vid.keys()]
    g.vs["id"] = [str(v) for v in pid_vid.keys()]
    g.add_edges(edges)

    if pid_comm_map is None:
        comm = g.community_leiden(objective_function="modularity")
        comm_to_csv(comm, g, f"comms/community-temp.csv")
        pid_comm_map = load_comm_map_communities("comms/community-temp.csv")


    g.vs["comm"] = [pid_comm_map[int(v["id"])] for v in g.vs]
    g.vs["color"] = [matplotlib.colors.to_hex(pal[v["comm"]]) for v in g.vs]

    # g = g.Read_Ncol("edges.csv", directed=False)
    del pid_vid
    del edges

    print("added edges")

    # full graph from 1000s -> 300s

    # g = comm
    # g = g.cluster_graph()
    print("found partitions")

    # for i, subgraph in enumerate(comm.subgraphs()):
    #     # count = 0
    #     # sorted_by_links = subgraph.vs.sort(key=lambda v: num_links_map[v["id"]])
    #     # print(sorted_by_links)
    #     # for v in subgraph.vs:
    #     #     v.delete()
    #     subgraph.clear()

    g.es["color"] = [matplotlib.colors.to_hex(pal[g.vs["comm"][e.source]]) for e in g.es]
    print(time.time() - t1, "seconds")
    #

    # print(len(comm), "communities found")

    # pal = ig.drawing.colors.PrecalculatedPalette(100)

    print(time.time() - t1, "seconds")
    return g

    # 100_000 => 162s
    # 150_000 => 240s???
    # 200_000 => 360s

    # 150_000_000 => 240_000s 66H???


def comm_to_csv(comm, g, out: str):
    with open(out, "w") as f:
        f.write("comm_id, page_id\n")
        for i, community in enumerate(comm):
            for v in community:
                f.write(f"{i},{g.vs[v]['id']}\n")


def plot_graph(g, layout):
    scale = 10
    ig.plot(g, "graph_top_all_cat_max_ref.png", layout=layout,
            # vertex_label=vertex_labels,
            # vertex_label=[str(comm.size(i)) for i, c in enumerate(comm)],
            vertex_label=g.vs["label"],
            # vertex_label_color=[get_fg(pal[c]) for c in g.vs["comm"]],
            vertex_label_size=[clamp(s, 10, 30) if s > 10 else 0 for s in g.vs["size"]],
            vertex_label_dist=0,
            # vertex_size=[clamp(math.log(comm.size(i))*10, 50, 250) for i, c in enumerate(comm)],
            vertex_size=g.vs["size"],
            # vertex_size=1,
            # vertex_color=[pal[c] for c in g.vs["comm"]],
            # vertex_color=[pal[i] for i, c in enumerate(g.vs)],
            # vertex_color=g.vs["color"],
            # edge_width=1,
            edge_size=1,
            # edge_arrow_size=1,
            # edge_color=[pal[g.vs["comm"][e.source]] for e in g.es],
            bbox=(1920 * scale, 1080 * scale)
            )
    plt.show()


def create_most_popular_graph(con, top: int, pid_comm_map):
    most_popular_pageids = [title_to_pid(con, title["article"]) for title in most_popular_pages() if
                            ":" not in title["article"]]

    pids = [str(pid) for pid in most_popular_pageids if pid is not None]
    pids = pids[:top]
    print("most pops:", len(pids))

    rows = con.execute(f"""SELECT page_id, page_link FROM WikiLink 
    where page_id in ({",".join(pids)})""").fetchall()
    print("fetched rows")

    # pid_comm_map = load_comm_map_communities("community-top-100.csv")
    g = create_graph(rows, con, pid_comm_map)

    # partition = la.find_partition(g, la.ModularityVertexPartition)
    # comm = g.community_edge_betweenness()
    # comm = comm.as_clustering()

    # layout = "fruchterman_reingold"
    # layout = "drl"

    layout = g.layout_drl()
    # print("made layout: " + str(time.time() - t1) + "s")
    # pickle.dump(layout, open("pages_top_popular_drl.layout", "wb"))

    # layout = pickle.load(open("pages_top_popular_drl.layout", "rb"))
    # print("loaded layout file")

    g.vs['x'], g.vs['y'] = zip(*layout.coords)
    return g

def create_top_graphs():
    bp = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/"

    top = 100
    wikis = ["dewiki", "enwiki"]
    for wiki_name in wikis:
        con = sqlite3.connect(f"{bp}/20240501/sqlite/{wiki_name}_database.sqlite")

        g = create_most_popular_graph(con, top, None)
        # comm = g.community_leiden(objective_function="modularity")
        # comm_to_csv(comm, g, f"comms/community-{wiki_name}-{top}.csv")
        #
        # g = create_most_popular_graph(con, top, load_comm_map_communities(f"comms/community-{wiki_name}-{top}.csv"))
        g.write_graphml(f"{bp}/graphs/{wiki_name}-most-popular-{top}.graphml")
        print(f"DONE: {wiki_name}")

# limit = 145_683_211
# limit = 100_000
# 100_000 => 1m
# 100_000_000 => 1000m

if __name__ == '__main__':
    create_top_graphs()
