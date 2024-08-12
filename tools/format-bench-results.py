import sys
import polars as pl

MAP_SIZE = 1000
SET_BUILD_FACTOR = 10 * MAP_SIZE

distr_order = [
    "u32",
    "u32pair",
    "u64",
    "u64lobits",
    "u64hibits",
    "u64pair",
    "ipv4",
    "ipv6",
    "rgba",
    "strenglishword",
    "struuid",
    "strurl",
    "strdate",
    "accesslog",
    "kilobyte",
    "tenkilobyte",
]

name_repl = {
    "foldhash-fast": "foldhash-f",
    "foldhash-quality": "foldhash-q",
}

bench_order = ["hashonly", "lookupmiss", "lookuphit", "setbuild"]
hash_order = ["foldhash-f", "foldhash-q", "fxhash", "ahash", "siphash"]

distr_order_df = pl.DataFrame({"distr": distr_order, "distr_order_idx": range(len(distr_order))})
bench_order_df = pl.DataFrame({"bench": bench_order, "bench_order_idx": range(len(bench_order))})
hash_order_df = pl.DataFrame({"hash": hash_order, "hash_order_idx": range(len(hash_order))})

df = (
    pl.scan_csv(sys.argv[1])
        .with_columns(pl.col.hash.replace(name_repl))
        .with_columns(ns = pl.col.ns / pl.when(pl.col.bench == "setbuild").then(SET_BUILD_FACTOR).otherwise(1))
        .join(distr_order_df.lazy(), on="distr")
        .join(bench_order_df.lazy(), on="bench")
        .join(hash_order_df.lazy(), on="hash")
        .sort(["distr_order_idx", "distr", "bench_order_idx", "hash_order_idx"])
        .select(pl.col.distr, pl.col.bench, pl.col.hash, pl.col.ns)
        .collect()
)

with pl.Config(tbl_rows=-1, float_precision=2, tbl_cell_alignment="RIGHT"):
    print(df.pivot("hash", values="ns"))
    print(
        df
            .with_columns(rank = pl.col.ns.rank().over("distr", "bench"))
            .group_by("hash", maintain_order=True)
            .agg(
                avg_rank = pl.col.rank.mean(),
                geometric_mean = pl.col.ns.log().mean().exp()
            )
            .transpose(include_header=True, header_name="metric", column_names="hash")
    )
