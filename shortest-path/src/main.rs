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
    // 距離行列
    let mut graph = [[MAX; NODE_NUM]; NODE_NUM];
    // 最短距離を格納する行列
    let mut dist_matrix = [[MAX; NODE_NUM]; NODE_NUM];
    // 経路を格納する行列
    let mut next_node = [[NODE_NUM; NODE_NUM]; NODE_NUM];

    // 始点ノード
    let mut source_node: usize = NODE_NUM;
    // 終点ノード
    let mut destination_node: usize = NODE_NUM;
    // ファイルから読み込むノードペアとその属性
    let mut node1: usize; // 接続元ノード
    let mut node2: usize; // 接続先ノード
    let mut distance: isize; // ノード間の距離
    let mut link_capacity: isize; // リンク容量

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

    // 距離行列の作成と初期化
    for i in 0..NODE_NUM {
        for j in 0..NODE_NUM {
            if i == j {
                graph[i][j] = 0;
                dist_matrix[i][j] = 0;
                next_node[i][j] = j;
            } else {
                graph[i][j] = MAX;
                dist_matrix[i][j] = MAX;
                next_node[i][j] = NODE_NUM;
            }
        }
    }

    // ファイルの内容を1行ずつ読み込んで処理
    for line in reader.lines() {
        match line {
            Ok(content) => {
                // 行の内容をスペースで分割
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
                    dist_matrix[node1][node2] = distance;
                    dist_matrix[node2][node1] = distance;
                    next_node[node1][node2] = node2;
                    next_node[node2][node1] = node1;
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

    // Floyd-Warshallアルゴリズムによる全点間最短経路の計算
    for k in 0..NODE_NUM {
        for i in 0..NODE_NUM {
            for j in 0..NODE_NUM {
                if dist_matrix[i][k] != MAX
                    && dist_matrix[k][j] != MAX
                    && dist_matrix[i][j] > dist_matrix[i][k] + dist_matrix[k][j]
                {
                    dist_matrix[i][j] = dist_matrix[i][k] + dist_matrix[k][j];
                    next_node[i][j] = next_node[i][k];
                }
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

    // CSVファイルを作成または開く
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
            communication_history.clear(); // 各シミュレーション実行前にクリア
            history_index = 0;
            for _communication_count in 0..MAX_ATTEMPTS {
                if !TEST_MODE {
                    source_node = rng.gen_range(0..NODE_NUM);
                    destination_node = rng.gen_range(0..NODE_NUM);
                    while source_node == destination_node {
                        destination_node = rng.gen_range(0..NODE_NUM);
                    }
                }

                // Dijkstraアルゴリズムの代わりに、事前計算した経路を使用
                if dist_matrix[source_node][destination_node] == MAX {
                    // 経路が存在しない場合の処理
                    if TEST_MODE {
                        println!(
                            "No path found from node{} to node{}.",
                            source_node, destination_node
                        );
                    }
                } else {
                    // 経路が存在する場合
                    let mut path = Vec::new();
                    let mut current = source_node;
                    while current != destination_node {
                        path.push(current);
                        current = next_node[current][destination_node];
                        if current == NODE_NUM {
                            break; // 経路が見つからない場合
                        }
                    }
                    path.push(destination_node);
                    // 通信経路上のリンク容量のチェック
                    let mut has_capacity = true;
                    for i in 0..path.len() - 1 {
                        let u = path[i];
                        let v = path[i + 1];
                        if bandwidth[u][v] < 1 {
                            has_capacity = false;
                            break;
                        }
                    }
                    total_attempt += 1;
                    if has_capacity {
                        // リンク容量を減少
                        for i in 0..path.len() - 1 {
                            let u = path[i];
                            let v = path[i + 1];
                            bandwidth[u][v] -= 1;
                            bandwidth[v][u] -= 1;
                        }
                        total_success += 1;
                        // communication_historyの更新
                        if communication_history.len() < n + 1 {
                            communication_history.push((true, path.clone()));
                        } else {
                            history_index %= n + 1;
                            communication_history[history_index] = (true, path.clone());
                            history_index += 1;
                        }
                    } else {
                        // 失敗した通信の記録
                        if communication_history.len() < n + 1 {
                            communication_history.push((false, Vec::new()));
                        } else {
                            history_index %= n + 1;
                            communication_history[history_index] = (false, Vec::new());
                            history_index += 1;
                        }
                    }
                    // nタイムユニット後にリンク容量を解放
                    if communication_history.len() > n {
                        let check_index = history_index % (n + 1);
                        if communication_history[check_index].0 {
                            // 通信が成功していた場合
                            let old_path = &communication_history[check_index].1;
                            for i in 0..old_path.len() - 1 {
                                let u = old_path[i];
                                let v = old_path[i + 1];
                                bandwidth[u][v] += 1;
                                bandwidth[v][u] += 1;
                            }
                        }
                    }
                }
            }

            let call_loss_rate = (total_attempt - total_success) as f64 / total_attempt as f64;
            println!("Call loss rate for run {}: {}", _run + 1, call_loss_rate);
            simulation_results.push(call_loss_rate);
        }

        // Calculate average
        let average = simulation_results.iter().sum::<f64>() / 10.0;
        println!("Average call loss rate for n = {}: {}", n + 1, average);
        // nと平均呼損率をCSVファイルに書き込みます
        writeln!(csv_file, "{},{}", n + 1, average).expect("Failed to write to CSV file");
    }
}
