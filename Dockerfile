# 使用官方Rust构建镜像
FROM rust:latest as builder

RUN apt update && apt upgrade -y && apt install -y protobuf-compiler libprotobuf-dev

# 为我们的应用创建一个目录
WORKDIR /usr/src/zkrpc

# 复制rust工程的所有文件到Docker镜像中的rust工程目录下
COPY . .

# 使用cargo构建我们的应用
RUN cargo install --path ./zkrpc

# 这种多阶段构建过程最后使用Debian图像
# 该阶段不用构建任何代码
FROM debian:latest


# 创建所需目录
RUN mkdir -p /root/.space-dev/config

# 从构建器阶段复制可执行的Rust应用并设置适当的权限。
COPY --from=builder /usr/local/cargo/bin/zkrpc /usr/local/bin
# 复制config.yaml文件到新建的文件夹下
COPY config/config.example.yaml /root/.space-dev/config/config.yaml

# 设置环境变量
ENV ENV=dev

# 配置服务运行
CMD ["zkrpc","server"]
