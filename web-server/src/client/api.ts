export const fetch_number_of_links = async (
    page_title: string,
    wiki_name: string,
) => {
    const resp = await fetch(`/api/${wiki_name}/${page_title}/links?num=true`);
    let json = await resp.json();
    //   console.log(json);

    return json;
};

export const fetch_number_times_linked = async (
    page_title: string,
    wiki_name: string,
) => {
    const resp = await fetch(`/api/${wiki_name}/${page_title}/linked?num=true`);
    return await resp.json();
};
