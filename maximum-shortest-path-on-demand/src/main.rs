use rand::Rng;
use std::collections::BinaryHeap;
use std::fs::OpenOptions;
use std::io::Write;
use std::process;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const NODE_NUM: usize = 10;
const MAX: isize = isize::MAX;
const MAX_ATTEMPTS: usize = 10000;

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

fn main() {
    let mut rng = rand::thread_rng();

    let file = Box::new(File::open("../distance.txt").expect("File not found"));
    let reader = BufReader::new(&*file);

    // 距離行列
    let mut graph = [[MAX; NODE_NUM]; NODE_NUM];
    // 前ノード
    let mut prev;
    // 距離

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
    let mut communication_history: Vec<(bool, Vec<usize>)> = Vec::with_capacity(10000);
    let mut history_index: usize;

    graph.iter_mut().enumerate().for_each(|(i, row)| {
        row.iter_mut().enumerate().for_each(|(j, val)| {
            if i == j {
                *val = 0;
                link[i][j] = -1;
            } else {
                *val = MAX;
                link[i][j] = -1;
            }
        });
    });

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

    for n in 0..10000 {
        println!("n = {}", n + 1);
        let mut simulation_results: Vec<f64> = Vec::new();

        for _run in 0..10 {
            bandwidth.iter_mut().enumerate().for_each(|(i, row)| {
                row.iter_mut().enumerate().for_each(|(j, val)| {
                    *val = link[i][j];
                });
            });
            total_success = 0;
            total_attempts = 0;
            communication_history.clear();
            history_index = 0;
            for _communication_count in 0..MAX_ATTEMPTS {
                source_node = rng.gen_range(0..NODE_NUM);
                destination_node = rng.gen_range(0..NODE_NUM);
                while source_node == destination_node {
                    destination_node = rng.gen_range(0..NODE_NUM);
                }

                // 経路探索のための変数を初期化
                prev = [usize::MAX; NODE_NUM];
                let mut max_capacity = [0; NODE_NUM];
                let mut heap = BinaryHeap::new();

                // 始点の設定
                max_capacity[source_node] = MAX;
                heap.push((max_capacity[source_node], source_node));

                // リンクを容量の大きい順にソート
                let mut sorted_links = Vec::new();
                bandwidth
                    .iter()
                    .enumerate()
                    .take(NODE_NUM)
                    .for_each(|(i, row)| {
                        row.iter()
                            .enumerate()
                            .skip(i + 1)
                            .take(NODE_NUM - i - 1)
                            .for_each(|(j, &val)| {
                                if val > 0 {
                                    sorted_links.push(LinkInfo {
                                        capacity: val,
                                        node1: i,
                                        node2: j,
                                    });
                                }
                            });
                    });
                sorted_links.sort_by(|a, b| b.capacity.cmp(&a.capacity));

                // サブグラフの初期化
                let mut subgraph = [[MAX; NODE_NUM]; NODE_NUM];
                let mut is_route_found = false;

                if !sorted_links.is_empty() {
                    let mut current_capacity = sorted_links[0].capacity;
                    let mut start_idx = 0;

                    for i in 0..=sorted_links.len() {
                        if i == sorted_links.len() || sorted_links[i].capacity != current_capacity {
                            for link_info in &sorted_links[start_idx..i] {
                                if bandwidth[link_info.node1][link_info.node2] > 0 {
                                    subgraph[link_info.node1][link_info.node2] =
                                        graph[link_info.node1][link_info.node2];
                                    subgraph[link_info.node2][link_info.node1] =
                                        graph[link_info.node2][link_info.node1];
                                }
                            }

                            // 経路探索（ダイクストラ法）
                            let mut dist = [MAX; NODE_NUM];
                            let mut prev = [usize::MAX; NODE_NUM];
                            let mut visited = [false; NODE_NUM];
                            dist[source_node] = 0;
                            let mut heap = std::collections::BinaryHeap::new();
                            heap.push(std::cmp::Reverse((0, source_node)));

                            while let Some(std::cmp::Reverse((cost_u, u))) = heap.pop() {
                                if visited[u] {
                                    continue;
                                }
                                visited[u] = true;
                                if u == destination_node {
                                    is_route_found = true;
                                    break;
                                }
                                for v in 0..NODE_NUM {
                                    if subgraph[u][v] != MAX && !visited[v] {
                                        let cost_v = cost_u + subgraph[u][v];
                                        if cost_v < dist[v] {
                                            dist[v] = cost_v;
                                            prev[v] = u;
                                            heap.push(std::cmp::Reverse((cost_v, v)));
                                        }
                                    }
                                }
                            }

                            if is_route_found {
                                break;
                            }

                            if i < sorted_links.len() {
                                current_capacity = sorted_links[i].capacity;
                                start_idx = i;
                            }
                        }
                    }
                } else {
                    is_route_found = false;
                }

                if is_route_found {
                    let mut path = Vec::new();
                    let mut node = destination_node;
                    while node != source_node && node != usize::MAX {
                        path.push(node);
                        node = prev[node];
                    }
                    path.push(source_node);
                    path.reverse();

                    for window in path.windows(2) {
                        if bandwidth[window[0]][window[1]] > 0 {
                            bandwidth[window[0]][window[1]] -= 1;
                            bandwidth[window[1]][window[0]] -= 1;
                        }
                    }

                    total_success += 1;
                    total_attempts += 1;

                    if communication_history.len() < n + 1 {
                        communication_history.push((true, path));
                    } else {
                        history_index %= n + 1;
                        communication_history[history_index] = (true, path);
                        history_index += 1;
                    }
                } else {
                    total_attempts += 1;
                    if communication_history.len() < n + 1 {
                        communication_history.push((false, Vec::new()));
                    } else {
                        history_index %= n + 1;
                        communication_history[history_index] = (false, Vec::new());
                        history_index += 1;
                    }
                }

                if communication_history.len() > n {
                    let check_index = history_index % (n + 1);
                    let (success, ref path) = &communication_history[check_index];
                    if *success && !path.is_empty() {
                        // 経路上のリンク容量を解放
                        for window in path.windows(2) {
                            if bandwidth[window[0]][window[1]] < link[window[0]][window[1]] {
                                bandwidth[window[0]][window[1]] += 1;
                                bandwidth[window[1]][window[0]] += 1;
                            }
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
        writeln!(csv_file, "{},{}", n + 1, average).expect("Failed to write to CSV file");
    }
}
