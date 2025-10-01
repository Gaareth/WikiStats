# wiki-stats-rs

Notes:
dewiki processing takes without download: ~8m 5s
dewiki stats processing 9s

[manjaro] jawiki download 20mb/s
[jawiki] --- WikiPage ---                                                                                                                                                                                                          Writing to database..                                                                                                                                                                                                              Successfully inserted data into db
Elapsed: 9.028525857s 
Speed: 265.099 rows/s

Creating unique index took: 1.144934514s

-- Total stats: --
Elapsed: 10.173529405s 
Speed: 235.263 rows/s

[jawiki] --- WikiLink ---                                                                                                                                                                                                          Writing to database..                                                                                                                                                                                                              Successfully inserted data into db
Elapsed: 89.045089315s 
Speed: 1.810.523 rows/s

Creating unique index took: 101.527529644s

-- Total stats: --
Elapsed: 190.57267267s 
Speed: 845.967 rows/s
[jawiki] Done. Total time elapsed: 206.852267362s                                                                                                                                                                                  
[jawiki] Done POSTINSERT. Total time elapsed: 118.752925236s




[wintermute] jawiki download 6mb/s? 
[jawiki] --- WikiPage ---                                                                                                                                                                      Writing to database..                                                                                                                                                                          Successfully inserted data into db
Elapsed: 9.766239572s 
Speed: 245.074 rows/s

Creating unique index took: 1.586096597s

-- Total stats: --
Elapsed: 11.352837102s 
Speed: 210.824 rows/s

-- Total stats: --
Elapsed: 365.658925182s 
Speed: 440.897 rows/s
[jawiki] Done. Total time elapsed: 383.183994647s 
[jawiki] Done POSTINSERT. Total time elapsed: 207.684322743s

Result: Manjaro is about two times faster. 
The downlaod is also faster but then transferring with rsync is likely slower in total

### Stats: 
manjaro: 8s
wintermute: 13s
