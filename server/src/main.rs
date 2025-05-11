use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let home = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./test/home.yaml"));

    // dir already requires GET...
    let files = warp::path("files").and(warp::fs::dir("./test/"));

    // GET / => home.yaml
    // GET /files/... => ./test/..
    let routes = home.or(files);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}