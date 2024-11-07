use rand::{thread_rng, Rng};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::process;

const NODE_NUM: usize = 10;
const MAX: isize = isize::MAX;
// Dijkstraのテストの場合は0に、シミュレーション評価を行う場合は1にする
const TEST_MODE: bool = false;
const MAX_ATTEMPTS: usize = 10000;

fn main() {
    // Dijkstraアルゴリズムで必要な変数
    // 距離行列
    let mut graph = [[MAX; NODE_NUM]; NODE_NUM];
    // 前ノード表
    let mut prev;
    // 距離を格納
    let mut dist;
    // 最短距離確定のフラグ
    let mut confirmed;

    // 始点ノード
    let mut source_node: usize = NODE_NUM;
    // 終点ノード
    let mut destination_node: usize = NODE_NUM;
    // ファイルから読み込むノードペアとその属性
    let mut node1: usize; // 接続元ノード
    let mut node2: usize; // 接続先ノード
    let mut distance: isize; // ノード間の距離
    let mut link_capacity: isize; // リンク容量
                                  // ループカウンタ
    let mut i: usize;

    // 経路探索完了フラグ
    let mut is_route_found;
    // ファイルを開く
    let file = Box::new(File::open("../distance.txt").expect("ファイルが開けません"));
    let reader = BufReader::new(&*file);

    // シミュレーション評価で必要な変数
    // リンク容量
    let mut link = [[-1; NODE_NUM]; NODE_NUM];
    // リンクの空き容量
    let mut bandwidth = [[-1; NODE_NUM]; NODE_NUM];

    // 確立できた通信回数の合計
    let mut total_success: usize;
    let mut total_attempt: usize;
    let mut communication_history: Vec<(bool, Vec<usize>)> = Vec::with_capacity(MAX_ATTEMPTS);
    let mut history_index: usize;

    // 距離行列の作成
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
                    // 文字列を数値に変換
                    node1 = values[0].parse::<usize>().unwrap_or(0);
                    node2 = values[1].parse::<usize>().unwrap_or(0);
                    distance = values[2].parse::<isize>().unwrap_or(0);
                    link_capacity = values[3].parse::<isize>().unwrap_or(0);

                    // グラフと容量の行列を更新
                    graph[node1][node2] = distance;
                    graph[node2][node1] = distance;
                    link[node1][node2] = link_capacity;
                    link[node2][node1] = link_capacity;
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                process::exit(1);
            }
        }
    }

    let mut rng = thread_rng();
    // 始点・終点ノードを設定
    if TEST_MODE {
        println!("Enter the source node:");
        let mut source_node_str = String::new();
        io::stdin()
            .read_line(&mut source_node_str)
            .expect("Failed to read line");
        source_node = source_node_str.trim().parse().unwrap();

        println!("Enter the destination node:");
        let mut destination_node_str = String::new();
        io::stdin()
            .read_line(&mut destination_node_str)
            .expect("Failed to read line");
        destination_node = destination_node_str.trim().parse().unwrap();
    }

    let mut csv_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("results.csv")
        .expect("Failed to open or create CSV file");

    // シミュレーション開始
    for n in 0..10000 {
        println!("\nSimulating for n = {}", n + 1);
        let mut simulation_results = Vec::new();

        for _run in 0..10 {
            // リンク容量を初期化
            for i in 0..NODE_NUM {
                for j in 0..NODE_NUM {
                    bandwidth[i][j] = link[i][j];
                }
            }
            total_success = 0;
            total_attempt = 0;
            communication_history.clear();
            history_index = 0;
            for _communication_count in 0..MAX_ATTEMPTS {
                if !TEST_MODE {
                    source_node = rng.gen_range(0..NODE_NUM);
                    destination_node = rng.gen_range(0..NODE_NUM);
                    while source_node == destination_node {
                        destination_node = rng.gen_range(0..NODE_NUM);
                    }
                }

                // Dijkstraアルゴリズム
                // 初期化
                dist = [MAX; NODE_NUM];
                confirmed = [false; NODE_NUM];
                prev = [usize::MAX; NODE_NUM];

                is_route_found = false;

                // 経路探索
                dist[source_node] = 0;
                prev[source_node] = source_node;

                while !is_route_found {
                    let mut min_dist = MAX;
                    let mut min_node: usize = NODE_NUM;

                    for i in 0..NODE_NUM {
                        if !confirmed[i] && dist[i] < min_dist {
                            min_dist = dist[i];
                            min_node = i;
                        }
                    }

                    if min_node == NODE_NUM {
                        break;
                    }

                    let current_node = min_node;
                    confirmed[current_node] = true;
                    if current_node == destination_node {
                        is_route_found = true;
                    }

                    for i in 0..NODE_NUM {
                        if !confirmed[i]
                            && graph[current_node][i] != MAX
                            && bandwidth[current_node][i] > 0
                            && dist[current_node] + graph[current_node][i] < dist[i]
                        {
                            dist[i] = dist[current_node] + graph[current_node][i];
                            prev[i] = current_node;
                        }
                    }
                }

                if is_route_found {
                    total_success += 1;
                    let mut path = Vec::new();
                    let mut current = destination_node;
                    while current != source_node {
                        path.push(current);
                        current = prev[current];
                    }
                    path.push(source_node);
                    path.reverse();

                    i = destination_node;
                    while i != source_node {
                        bandwidth[i][prev[i]] -= 1;
                        bandwidth[prev[i]][i] -= 1;
                        i = prev[i];
                    }
                    if communication_history.len() < n + 1 {
                        communication_history.push((true, path));
                    } else {
                        history_index %= n + 1;
                        if history_index < communication_history.len() {
                            communication_history[history_index] = (true, path);
                        }
                        history_index += 1;
                    }
                } else if communication_history.len() < n + 1 {
                    communication_history.push((false, Vec::new()));
                } else {
                    history_index %= n + 1;
                    if history_index < communication_history.len() {
                        communication_history[history_index] = (false, Vec::new());
                    }
                    history_index += 1;
                }

                total_attempt += 1;

                // リンク容量を解放
                if communication_history.len() > n {
                    let check_index = history_index % (n + 1);
                    if check_index < communication_history.len() {
                        let (success, ref path) = &communication_history[check_index];
                        if *success && path.len() >= 2 {
                            // Release capacity along the stored path
                            for window in path.windows(2) {
                                if bandwidth[window[0]][window[1]] < link[window[0]][window[1]] {
                                    bandwidth[window[0]][window[1]] += 1;
                                }
                                if bandwidth[window[1]][window[0]] < link[window[1]][window[0]] {
                                    bandwidth[window[1]][window[0]] += 1;
                                }
                            }
                        }
                    }
                }
            }

            let call_loss_rate = (total_attempt - total_success) as f64 / total_attempt as f64;
            println!("Call loss rate for run {}: {}", _run + 1, call_loss_rate);
            simulation_results.push(call_loss_rate);
        }

        let average = simulation_results.iter().sum::<f64>() / 10.0;
        println!("Average call loss rate for n = {}: {}", n + 1, average);
        writeln!(csv_file, "{},{}", n + 1, average).expect("Failed to write to CSV file");
    }
}
