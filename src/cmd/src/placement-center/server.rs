// use clap::command;
use clap::Parser;


// 定义默认的配置路径，即当命令行没传配置路径时，默认的配置文件路径
pub const DEFAULT_PLACEMENT_CENTER_CONFIG: &str = "config/placement-center.toml";


// 定义接收哪些参数
#[derive(Parser, Debug)]
#[command(author="robustmq-geek", version="0.0.1", about=" RobustMQ: Next generation cloud-native converged high-performance message queue.", long_about = None)]
#[command(next_line_help = true)]
struct ArgsParams {
    #[arg(short, long, default_value_t=String::from(DEFAULT_PLACEMENT_CENTER_CONFIG))]
    conf: String,
}


fn main() {
    // 解析命令行参数
    let args = ArgsParams::parse();
    println!("conf path: {:?}", args.conf);
}