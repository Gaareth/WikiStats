use std::{collections::HashMap, hash::BuildHasher, path::PathBuf};

use fxhash::{FxHashMap, FxHashSet};
use parse_mediawiki_sql::field_types::PageId;

use crate::{stats::queries::top_linked_ids, utils::default_bar};

fn floyd_warshall<S: BuildHasher>(cache: &HashMap<PageId, Vec<PageId>, S>) {
    // let mut dist: Vec<Vec<u8>> = (0..4464087)
    // .map(|_| (0..4464087).map(|_| 0).collect())
    // .collect();
    let mut dist: FxHashMap<u32, FxHashMap<u32, u32>> = FxHashMap::default();
    // dbg!(&dist[0]);
    // return

    let all_ids = get_all_ids(cache);
    let num_ids = all_ids.len();
    println!("There are {num_ids} ids");

    //
    // for vertex in all_ids.iter() {
    //     // dist.get(vertex.0 as usize).unwrap_or(vec![])[vertex.0 as usize] = 0;
    //     dist.get()
    // }
    //
    // println!("initialized self edges");

    for (u, v) in get_all_edges(cache) {
        let uid = u.0;
        let vid = v.0;

        if dist.get(&uid).is_none() {
            let neighbours = FxHashMap::default();
            dist.insert(uid, neighbours);
        }
        let neighbours = dist.get_mut(&uid).unwrap();
        neighbours.insert(vid, 1);
    }
    println!("initialized other edges");

    let db_path: PathBuf = todo!();
    let subset = top_linked_ids(5, Some("de"), db_path);
    dbg!(&subset.len());

    let bar = default_bar((subset.len().pow(2)) as u64);
    for k in subset.iter() {
        for i in subset.iter() {
            for j in subset.iter() {
                let dist_i_j = get_2d(i.0, j.0, &mut dist);
                let dist_i_k = get_2d(i.0, k.0, &mut dist);
                let dist_k_j = get_2d(k.0, j.0, &mut dist);

                if dist_i_j > (dist_i_k + dist_k_j) {
                    dist.get_mut(&(i.0))
                        .unwrap()
                        .insert(j.0, (dist_i_k + dist_k_j));
                    // dist[i][j] = dist[i][k] + dist[k][j]
                }
            }
            bar.inc(1);
        }
    }
    //  let start_link = PageTitle("Auto_(Begriffsklärung)".to_string());
    // let start_link_id = sqlite::title_id_conv::page_title_to_id(&start_link).unwrap();
    //
    // let end_link = PageTitle("Kuba".to_string());
    //  let end_link_id = sqlite::title_id_conv::page_title_to_id(&end_link).unwrap();

    dbg!(&dist.get(&6000499).unwrap().get(&5782088).unwrap());
    bar.finish();
}

fn get_2d(i: u32, j: u32, map: &mut FxHashMap<u32, FxHashMap<u32, u32>>) -> u32 {
    // let dist_i_j = map.entry(i as u32)
    //                .or_insert(FxHashMap::default()).get(&(j as u32)).unwrap_or(&u32::MAX);
    if map.get(&i).is_none() {
        map.insert(i, FxHashMap::default());
    }
    *map.get(&i).unwrap().get(&j).unwrap_or(&u32::MAX)
}

fn get_all_edges<S: BuildHasher>(
    cache: &HashMap<PageId, Vec<PageId>, S>,
) -> FxHashSet<(PageId, PageId)> {
    let mut edges: FxHashSet<(PageId, PageId)> = FxHashSet::default();
    for (id, links) in cache {
        for edge in links {
            edges.insert((*id, *edge));
        }
    }
    return edges;
}

fn get_all_ids<S: BuildHasher>(cache: &HashMap<PageId, Vec<PageId>, S>) -> FxHashSet<PageId> {
    let mut ids = FxHashSet::default();
    for (id, links) in cache {
        ids.insert(*id);
        ids.extend(links);
    }
    return ids;
}
