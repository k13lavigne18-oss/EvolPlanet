FROM rust:latest

# 1. 必要なパッケージのインストール
RUN apt-get update && apt-get install -y \
    g++ pkg-config libx11-dev libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev \
    xvfb x11vnc net-tools python3-numpy \
    openbox supervisor \
    mesa-vulkan-drivers libgl1-mesa-dri libglx-mesa0 \
    git \
    && rm -rf /var/lib/apt/lists/*

# 2. noVNC のセットアップ
RUN git clone https://github.com/novnc/noVNC.git /opt/novnc \
    && git clone https://github.com/novnc/websockify /opt/novnc/utils/websockify \
    && ln -s /opt/novnc/vnc.html /opt/novnc/index.html

WORKDIR /app
COPY . .

# 3. ゲームのビルド
RUN cargo build --release

# 【重要】バイナリ名が "_" か "-" か分からなくても大丈夫なように、
#  見つかった方を /app/game にリネームして配置します。
RUN if [ -f target/release/evolution_game ]; then \
        cp target/release/evolution_game /app/game; \
    elif [ -f target/release/evolution-game ]; then \
        cp target/release/evolution-game /app/game; \
    else \
        # 万が一どちらも見つからない場合はビルド失敗とみなす
        echo "Binary not found!" && exit 1; \
    fi

# 4. Supervisor設定 (起動管理)
RUN echo "[supervisord]" > /etc/supervisor/conf.d/supervisord.conf && \
    echo "nodaemon=true" >> /etc/supervisor/conf.d/supervisord.conf && \
    # Xvfb: 仮想ディスプレイ
    echo "[program:xvfb]" >> /etc/supervisor/conf.d/supervisord.conf && \
    echo "command=/usr/bin/Xvfb :1 -screen 0 1280x720x24" >> /etc/supervisor/conf.d/supervisord.conf && \
    # Openbox: ウィンドウマネージャー
    echo "[program:openbox]" >> /etc/supervisor/conf.d/supervisord.conf && \
    echo "command=/usr/bin/openbox-session" >> /etc/supervisor/conf.d/supervisord.conf && \
    echo "environment=DISPLAY=:1" >> /etc/supervisor/conf.d/supervisord.conf && \
    # x11vnc: VNCサーバー
    echo "[program:x11vnc]" >> /etc/supervisor/conf.d/supervisord.conf && \
    echo "command=/usr/bin/x11vnc -display :1 -nopw -listen localhost -xkb -ncache 10 -ncache_cr -forever" >> /etc/supervisor/conf.d/supervisord.conf && \
    # noVNC: Webプロキシ
    echo "[program:novnc]" >> /etc/supervisor/conf.d/supervisord.conf && \
    echo "command=/opt/novnc/utils/novnc_proxy --vnc localhost:5900 --listen 8080" >> /etc/supervisor/conf.d/supervisord.conf && \
    # Game: ゲーム本体 (リネームした /app/game を実行)
    echo "[program:game]" >> /etc/supervisor/conf.d/supervisord.conf && \
    echo "environment=DISPLAY=:1,WGPU_BACKEND=vulkan,LIBGL_ALWAYS_SOFTWARE=1" >> /etc/supervisor/conf.d/supervisord.conf && \
    # ここを修正: /app/game を実行するように変更
    echo "command=/app/game" >> /etc/supervisor/conf.d/supervisord.conf

EXPOSE 8080
CMD ["/usr/bin/supervisord"]