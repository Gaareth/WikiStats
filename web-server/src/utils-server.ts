import { fetchSiteMatrix, type SiteInfo } from "./wiki-api";

export const DBNAME_TO_SITEINFO = await (async () => {
    const dbname_to_siteinfo = new Map<string, SiteInfo>();

    const site_matrix = await fetchSiteMatrix();
    for (const key in site_matrix) {
        if (!isNaN(Number(key))) {
            const data = site_matrix[key];
            for (const site of data.site) {
                dbname_to_siteinfo.set(site.dbname, {
                    url: site.url,
                    dbname: site.dbname,
                    code: data.code,
                    sitename: site.sitename,
                    name: data.name,
                    localname: data.localname,
                });
            }
        } else if (key === "specials") {
            for (const special_key in site_matrix.specials) {
                const site = site_matrix.specials[special_key];
                dbname_to_siteinfo.set(site.dbname, {
                    url: site.url,
                    dbname: site.dbname,
                    code: site.code,
                    sitename: site.sitename,
                });
            }
        }
    }
    return dbname_to_siteinfo;
})();