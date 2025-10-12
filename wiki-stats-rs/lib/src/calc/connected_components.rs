use std::fs;

use fxhash::FxHashSet;
use log::info;
use parse_mediawiki_sql::field_types::PageId;

use crate::{
    WikiIdent,
    calc::bfs::bfs_undirected,
    sqlite::{page_links::get_cache, title_id_conv::load_rows_from_page},
    stats::queries::query_page,
    utils::ProgressBarBuilder,
};

pub fn find_wcc(wiki_ident: WikiIdent) {
    let db_path = wiki_ident.clone().db_path;

    let all_pages = load_rows_from_page(&db_path)
        .into_iter()
        .map(|p| p.0.0)
        .collect::<FxHashSet<_>>();
    let redirects = query_page(
        "SELECT * FROM WikiPage WHERE is_redirect = 1;",
        &db_path,
        wiki_ident.wiki_name.clone(),
    )
    .into_iter()
    .map(|p| p.page_id as u32)
    .collect::<FxHashSet<_>>();

    let mut components: Vec<FxHashSet<u32>> = Vec::new();
    let mut visited: FxHashSet<u32> = FxHashSet::default();
    let cache = get_cache(&db_path, None, false);
    let incoming_cache = get_cache(&db_path, None, true);

    let bar = ProgressBarBuilder::new()
        .with_name("Finding WCC")
        .with_length(all_pages.len() as u64)
        .build();

    for page in all_pages {
        if !visited.contains(&page) {
            let connected_component: FxHashSet<u32> =
                bfs_undirected(&PageId(page), &cache, &incoming_cache, db_path.clone())
                    .into_iter()
                    .map(|pid| pid.0)
                    .collect();

            // bar.println(format!("Found WCC: {:?}", connected_component.len()));
            // bar.inc(connected_component.len() as u64);

            components.push(connected_component.clone());
            visited.extend(connected_component);
        }
        bar.inc(1);
    }
    info!("Num components: {}", components.len());

    // remove all redirects
    components.retain_mut(|c| {
        c.retain(|p| !redirects.contains(p));
        c.len() > 1
    });

    components.sort_by(|a, b| a.len().cmp(&b.len())); // ascending
    components.pop(); // remove last, so largest component
    // todo: also remove Begriffsklärungsseiten

    info!("Num components after pruning: {}", components.len());

    let path = "components.json";
    let json = serde_json::to_string_pretty(&components).unwrap();
    fs::write(&path, json).expect(&format!("Failed writing stats to file {}", path));
}

pub fn find_scc(wiki_ident: WikiIdent) {
    todo!()
}
