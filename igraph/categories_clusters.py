import math
import sqlite3
import time

from tqdm import tqdm

con = sqlite3.connect("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240401/dewiki_cl_database.sqlite")
con_title_id_all = sqlite3.connect(
    "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240401/dewiki_title_id_all_database.sqlite")

con_title_id = sqlite3.connect(
    "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240401/dewiki_title_id_database.sqlite")

rows = con.execute(f"""SELECT page_id_from, category_name
FROM WikiCategoryLinks where category_type == 'page'""").fetchall()
print("rows loaded")

rows_subcat = con.execute(f"""SELECT page_id_from, category_name
FROM WikiCategoryLinks where category_type == 'subcat'""")


def shortest_way_up(subcat, category_to_parent_categories, count=0, visited=None):
    if visited is None:
        visited = [subcat]

    # print(subcat)
    if subcat == "!Hauptkategorie":
        return count

    for parent in category_to_parent_categories[subcat]:
        if parent not in visited:
            visited.append(parent)
            return shortest_way_up(parent, category_to_parent_categories, count=count + 1, visited=visited)


def query_id_titlemap(con):
    id_title_map = {}
    for (page_id, page_title) in con.execute("""SELECT page_id, page_title FROM WikiPage"""):
        id_title_map[page_id] = page_title
    return id_title_map


id_title_map_all = query_id_titlemap(con_title_id_all)
id_title_map = query_id_titlemap(con_title_id)

limit = 16_242_148
t1 = time.time()


def calc_page_to_categories_map(rows):
    page_to_categories = {}
    for (page_id, target_category_name) in tqdm(rows, total=limit):
        source_page_name = id_title_map_all[page_id]
        if source_page_name not in page_to_categories:
            page_to_categories[source_page_name] = []

        page_to_categories[source_page_name].append(target_category_name)

    print("pages with categories: ", len(page_to_categories))
    return page_to_categories


def get_categories():
    category_pages = {}
    for (page_id, target_category_name) in tqdm(rows, total=limit):
        source_page_name = id_title_map_all[page_id]
        if target_category_name not in category_pages:
            category_pages[target_category_name] = []

        category_pages[target_category_name].append(source_page_name)

    print("categories: ", len(category_pages))
    # for c1 in tqdm(category_pages.keys(), total=len(category_pages)):
    #     for c2 in category_pages.keys():
    #         if c1 != c2 and category_pages[c1] == category_pages[c2]:
    #             print(c1)
    #             print(c2)
    return category_pages


def reduce_dups():
    pass
    # for c1 in page_categories:
    #     for c2 in page_categories:
    #         if c1 != c2 and category_pages[c1] == category_pages[c2]:
    #             # category_pages[c1 + c2] = category_pages[c1]
    #             print(c1 + " and " + c2 + " are eq")
    #             print(page_categories)
    #             print("\n")
    #
    #             page_categories.remove(c1)
    #             page_categories.remove(c2)
    #
    #             new_cat = c1 + " | " + c2
    #             page_categories.append(new_cat)
    #
    #             print(page_categories)
    #
    #             # category_pages[new_cat] = category_pages[c1]
    #             # category_pages[new_cat].extend(category_pages[c2])


def check_by_pages():
    category_pages = get_categories()
    page_to_categories_map = calc_page_to_categories_map(rows)
    category_to_parent_categories = calc_page_to_categories_map(rows_subcat)

    equal_cats = set()

    page_to_category_map = {}

    page_to_len_path_up = {}

    categories = set()
    category_count = {}
    category_len_hist = {}

    for page_title in tqdm(id_title_map.values()):
        if page_title not in page_to_categories_map:
            # likely redirect without categories
            continue
        page_categories = page_to_categories_map[page_title]

        max_depth = (0, "")
        min_depth = (math.inf, "")

        max_ref = (0, "")

        for cat in page_categories:
            if cat.startswith("Wikipedia:"):
                continue

            if len(category_pages[cat]) > max_ref[0]:
                max_ref = (len(category_pages[cat]), cat)

        # if cat not in page_to_len_path_up:
            #     try:
            #         page_to_len_path_up[cat] = shortest_way_up(cat, category_to_parent_categories)
            #     except KeyError:
            #         continue
            #
            # # print(f"{cat}: {page_to_len_path_up[cat]}")
            # if page_to_len_path_up[cat] > max_depth[0]:
            #     max_depth = (page_to_len_path_up[cat], cat)
            #
            # if page_to_len_path_up[cat] < min_depth[0]:
            #     min_depth = (page_to_len_path_up[cat], cat)

        if max_ref[0] != 0:
            cat = max_ref[1]
            page_to_category_map[page_title] = cat
            categories.add(cat)

            if cat not in category_count:
                category_count[cat] = 0
            category_count[cat] += 1

        # print(max_depth)
        # min depth 155_761
        # max depth 249_589
        # max ref 79_001
        # print(page_categories)
    print("categories: ", len(categories))

    for (cat, num_items) in category_count.items():

        if num_items == 1:
            print(cat)

        if num_items not in category_len_hist:
            category_len_hist[num_items] = 0
        category_len_hist[num_items] += 1

    print(dict(sorted(category_len_hist.items())))
    # print(page_to_category_map)
    # print(len(equal_cats))
    with open("category_communities_max_ref.csv", "w") as f:
        f.write("category_name, page_title\n")
        for page_title, category in page_to_category_map.items():
            f.write(f"{category}, {page_title}\n")


check_by_pages()
# category_to_parent_categories = calc_page_to_categories_map(rows_subcat)
# print(shortest_way_up("Haltung_von_Schafen_und_Ziegen", category_to_parent_categories))

print((time.time() - t1), "s")
