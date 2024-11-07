use rand::Rng;
use std::fs::OpenOptions;
use std::io::Write;
use std::process;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const NODE_NUM: usize = 10;
const MAX: isize = isize::MAX;
const TEST_MODE: bool = false;
const MAX_ATTEMPTS: usize = 10000;

// 経路情報を保持する構造体
#[derive(Clone)]
struct PathInfo {
    prev: [usize; NODE_NUM],
    dist: [isize; NODE_NUM],
}

// 全ノード間の経路情報を保持する構造体
struct AllPathsInfo {
    paths: Vec<Vec<PathInfo>>,
}

impl AllPathsInfo {
    fn new() -> Self {
        let paths = vec![
            vec![
                PathInfo {
                    prev: [NODE_NUM; NODE_NUM],
                    dist: [MAX; NODE_NUM],
                };
                NODE_NUM
            ];
            NODE_NUM
        ];
        AllPathsInfo { paths }
    }

    fn get_path(&self, source: usize, dest: usize) -> &PathInfo {
        &self.paths[source][dest]
    }
}

// 全ノード間の最大最小距離を計算する関数
fn calculate_all_paths(
    sorted_links: &[LinkInfo],
    graph: &[[isize; NODE_NUM]; NODE_NUM],
) -> AllPathsInfo {
    let mut all_paths = AllPathsInfo::new();

    for source in 0..NODE_NUM {
        for dest in 0..NODE_NUM {
            if source != dest {
                let (prev, dist) = find_maximum_capacity_path(sorted_links, source, dest, graph);
                all_paths.paths[source][dest] = PathInfo { prev, dist };
            }
        }
    }

    all_paths
}

#[derive(Debug, Eq, PartialEq)]
struct LinkInfo {
    capacity: isize,
    node1: usize,
    node2: usize,
}

impl Ord for LinkInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.capacity.cmp(&self.capacity)
    }
}

impl PartialOrd for LinkInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn find_maximum_capacity_path(
    sorted_links: &[LinkInfo],
    source_node: usize,
    destination_node: usize,
    graph: &[[isize; NODE_NUM]; NODE_NUM],
) -> ([usize; NODE_NUM], [isize; NODE_NUM]) {
    let mut subgraph = [[MAX; NODE_NUM]; NODE_NUM];
    let mut prev = [NODE_NUM; NODE_NUM];
    let mut dist = [MAX; NODE_NUM];

    let mut current_capacity = sorted_links[0].capacity;
    let mut start_idx = 0;
    let mut is_route_found = false;

    for i in 0..=sorted_links.len() {
        // 最後まで到達するか、容量が変わった場合
        if i == sorted_links.len() || sorted_links[i].capacity != current_capacity {
            for link_info in &sorted_links[start_idx..i] {
                subgraph[link_info.node1][link_info.node2] =
                    graph[link_info.node1][link_info.node2];
                subgraph[link_info.node2][link_info.node1] =
                    graph[link_info.node2][link_info.node1];
            }

            // 探索のための変数を初期化
            dist = [MAX; NODE_NUM];
            let mut confirmed = [false; NODE_NUM];
            prev = [NODE_NUM; NODE_NUM];

            dist[source_node] = 0;
            prev[source_node] = source_node;

            // ダイクストラ法による最短経路探索
            while !is_route_found {
                // 確定していないノードの中から最小距離のノードを選択
                let mut min_dist = MAX;
                let mut min_node = NODE_NUM;
                for j in 0..NODE_NUM {
                    if !confirmed[j] && dist[j] < min_dist {
                        min_dist = dist[j];
                        min_node = j;
                    }
                }

                // これ以上進めない場合は終了
                if min_node == NODE_NUM {
                    break;
                }

                let current_node = min_node;
                confirmed[current_node] = true;

                // 終点に到達した場合
                if current_node == destination_node {
                    is_route_found = true;
                    break;
                }

                // 隣接ノードの距離を更新
                for j in 0..NODE_NUM {
                    if !confirmed[j]
                        && subgraph[current_node][j] != MAX
                        && dist[current_node] + subgraph[current_node][j] < dist[j]
                    {
                        dist[j] = dist[current_node] + subgraph[current_node][j];
                        prev[j] = current_node;
                    }
                }
            }

            // ルートが見つかった場合、ループを抜ける
            if is_route_found {
                break;
            }

            if i < sorted_links.len() {
                current_capacity = sorted_links[i].capacity;
                start_idx = i;
            }
        }
    }

    (prev, dist)
}

// 通信履歴を保持する構造体
#[derive(Clone)]
struct CommunicationRecord {
    success: bool,
    source: usize,
    destination: usize,
    path: Vec<usize>,
}

fn main() {
    let mut rng = rand::thread_rng();

    let file = Box::new(File::open("../distance.txt").expect("File not found"));
    let reader = BufReader::new(&*file);

    // 距離行列
    let mut graph = [[MAX; NODE_NUM]; NODE_NUM];

    let mut source_node: usize;
    let mut destination_node: usize;

    let mut node1: usize;
    let mut node2: usize;
    let mut distance: isize;
    let mut link_capacity: isize;

    // リンク容量
    let mut link = [[-1; NODE_NUM]; NODE_NUM];
    // リンクの空き容量
    let mut bandwidth = [[-1; NODE_NUM]; NODE_NUM];

    let mut total_success: usize;
    let mut total_attempts;
    let mut communication_history: Vec<CommunicationRecord> = Vec::with_capacity(10000);
    let mut history_index: usize;

    for i in 0..NODE_NUM {
        for j in 0..NODE_NUM {
            if i == j {
                graph[i][j] = 0;
                link[i][j] = -1;
            } else {
                graph[i][j] = MAX;
                link[i][j] = -1;
            }
        }
    }

    for line in reader.lines() {
        match line {
            Ok(content) => {
                let values: Vec<&str> = content.split_whitespace().collect();
                if values.len() == 4 {
                    node1 = values[0].parse::<usize>().unwrap();
                    node2 = values[1].parse::<usize>().unwrap();
                    distance = values[2].parse::<isize>().unwrap();
                    link_capacity = values[3].parse::<isize>().unwrap();

                    graph[node1][node2] = distance;
                    graph[node2][node1] = distance;
                    link[node1][node2] = link_capacity;
                    link[node2][node1] = link_capacity;
                }
            }
            Err(e) => {
                println!("Error reading line: {}", e);
                process::exit(1);
            }
        }
    }

    let mut csv_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("results.csv")
        .expect("Failed to open or create CSV file");

    writeln!(csv_file, "n,average_call_loss_rate").expect("Failed to write to CSV file");

    // グラフ上のリンクを重みの大きい順にソート
    let mut sorted_links = Vec::new();
    link.iter().enumerate().take(NODE_NUM).for_each(|(i, row)| {
        row.iter()
            .enumerate()
            .skip(i + 1)
            .take(NODE_NUM - i - 1)
            .for_each(|(j, &capacity)| {
                if capacity > -1 {
                    sorted_links.push(LinkInfo {
                        capacity,
                        node1: i,
                        node2: j,
                    });
                }
            });
    });
    sorted_links.sort();

    let all_paths = calculate_all_paths(&sorted_links, &graph);

    if TEST_MODE {
        let source_node = 0;
        let destination_node = NODE_NUM - 1;
        let path_info = all_paths.get_path(source_node, destination_node);

        println!(
            "The shortest path from node{} to node{} is:",
            source_node, destination_node
        );
        println!("{} <-", destination_node);
        let mut node = destination_node;
        while path_info.prev[node] != source_node {
            print!("{} <-", path_info.prev[node]);
            node = path_info.prev[node];
        }
        println!("{}.", source_node);
        println!("The distance is {}.", path_info.dist[destination_node]);
        return;
    }

    for n in 0..10000 {
        println!("n = {}", n + 1);
        let mut simulation_results: Vec<f64> = Vec::new();

        for _run in 0..10 {
            for i in 0..NODE_NUM {
                for j in 0..NODE_NUM {
                    bandwidth[i][j] = link[i][j];
                }
            }
            total_success = 0;
            total_attempts = 0;
            communication_history.clear();
            history_index = 0;
            for _ in 0..MAX_ATTEMPTS {
                source_node = rng.gen_range(0..NODE_NUM);
                destination_node = rng.gen_range(0..NODE_NUM);
                while source_node == destination_node {
                    destination_node = rng.gen_range(0..NODE_NUM);
                }

                let path_info = all_paths.get_path(source_node, destination_node);

                // 経路を保存
                let mut path = Vec::new();
                let mut node = destination_node;
                path.push(node);
                while node != source_node {
                    node = path_info.prev[node];
                    path.push(node);
                }
                path.reverse();

                // 容量チェック
                let mut has_capacity = true;
                for window in path.windows(2) {
                    if bandwidth[window[0]][window[1]] < 1 {
                        has_capacity = false;
                        break;
                    }
                }

                total_attempts += 1;

                if has_capacity {
                    // 容量を減少
                    for window in path.windows(2) {
                        bandwidth[window[0]][window[1]] -= 1;
                        bandwidth[window[1]][window[0]] -= 1;
                    }
                    total_success += 1;

                    // 通信記録を保存
                    if communication_history.len() < n + 1 {
                        communication_history.push(CommunicationRecord {
                            success: true,
                            source: source_node,
                            destination: destination_node,
                            path,
                        });
                    } else {
                        history_index = total_attempts % (n + 1);
                        if history_index < communication_history.len() {
                            communication_history[history_index] = CommunicationRecord {
                                success: true,
                                source: source_node,
                                destination: destination_node,
                                path,
                            };
                        }
                    }
                } else if communication_history.len() < n + 1 {
                    communication_history.push(CommunicationRecord {
                        success: false,
                        source: source_node,
                        destination: destination_node,
                        path: vec![],
                    });
                } else {
                    history_index = total_attempts % (n + 1);
                    if history_index < communication_history.len() {
                        communication_history[history_index] = CommunicationRecord {
                            success: false,
                            source: source_node,
                            destination: destination_node,
                            path: vec![],
                        };
                    }
                }

                // n回前の通信を解放
                if communication_history.len() > n {
                    let check_index = total_attempts % (n + 1);
                    if check_index < communication_history.len()
                        && communication_history[check_index].success
                    {
                        let old_path = &communication_history[check_index].path;
                        for window in old_path.windows(2) {
                            bandwidth[window[0]][window[1]] += 1;
                            bandwidth[window[1]][window[0]] += 1;
                        }
                    }
                }
            }
            let call_loss_rate = (total_attempts - total_success) as f64 / total_attempts as f64;
            println!("Call loss rate for run {}: {}", _run + 1, call_loss_rate);
            simulation_results.push(call_loss_rate);
        }

        let average = simulation_results.iter().sum::<f64>() / simulation_results.len() as f64;
        println!("Average call loss rate for n = {}: {}", n + 1, average);
        // nと平均呼損率をCSVファイルに書き込みます
        writeln!(csv_file, "{},{}", n + 1, average).expect("Failed to write to CSV file");
    }
}
