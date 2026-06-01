#!/bin/sh
set -e
echo "============================================="
echo " MQTT订阅服务 一键部署"
echo "============================================="

# ==================== 核心配置 ====================
APP_NAME="som-rmqtt-sub"
LOCAL_CONFIG="config.json5"
REMOTE_CONFIG_URL="https://github.com/SomunsMo/som-rmqtt-sub/raw/master/config/config.json5"
BIN_DIR="/usr/bin"
APP_LOCAL_BIN="./${APP_NAME}"
APP_BIN="${BIN_DIR}/${APP_NAME}"
CONFIG_DIR="/etc/${APP_NAME}"
CONFIG_FILE="${CONFIG_DIR}/${LOCAL_CONFIG}"
LOG_DIR="/var/log/${APP_NAME}"
# 启动脚本放应用配置目录
SERVICE_WRAPPER="${CONFIG_DIR}/wrapper.sh"
INIT_SERVICE="/etc/init.d/${APP_NAME}"

# ==================== 创建目录 ====================
echo "-- 初始化系统目录..."
mkdir -p "${BIN_DIR}" "${CONFIG_DIR}" "${LOG_DIR}"
chmod 755 "${BIN_DIR}" "${CONFIG_DIR}" "${LOG_DIR}"

# ==================== 检测主程序 ====================
echo "-- 检测主程序 ${APP_NAME}..."
# 本地有主程序 → 覆盖部署
if [ -f "${APP_LOCAL_BIN}" ]; then
    echo "脚本运行目录下检测到主程序，将覆盖部署。"
    echo "- 主程序校验通过"

    # 部署主程序
    echo "- 部署主程序到 ${APP_BIN}..."
    mv -f "${APP_LOCAL_BIN}" "${APP_BIN}"
    chmod 755 "${APP_BIN}"
    echo "主程序部署完成"

# 本地无主程序，检查目标位置
else
    if [ -f "${APP_BIN}" ]; then
        echo "脚本运行目录下无主程序文件，但部署目标位置已存在主程序，跳过主程序部署环节。"
    else
        echo "Err：脚本运行目录与部署目标位置均未找到主程序 ${APP_NAME}，安装终止！"
        exit 1
    fi
fi

# ==================== 配置文件处理 ====================
echo "-- 处理配置文件..."
if [ -f "${CONFIG_FILE}" ]; then
    echo "目标路径已存在配置文件 ${CONFIG_FILE}，跳过配置文件处理环节（避免覆盖原有配置）。"
else
    if [ -f "./${LOCAL_CONFIG}" ]; then
        echo "检测到本地配置文件，部署到 ${CONFIG_FILE}。"
        mv -f "./${LOCAL_CONFIG}" "${CONFIG_FILE}"
        chmod 644 "${CONFIG_FILE}"
    else
        echo "未找到本地配置，开始下载默认配置..."
        wget -q --no-check-certificate -O "${CONFIG_FILE}" "${REMOTE_CONFIG_URL}"
        if [ ! -f "${CONFIG_FILE}" ]; then
            echo "Err：配置文件下载失败，请检查网络！安装终止！"
            exit 1
        fi
        chmod 666 "${CONFIG_FILE}"
        echo "- 配置文件下载完成"
    fi
fi

# ==================== 生成启动脚本 ====================
echo "-- 生成启动包装脚本..."
cat > "${SERVICE_WRAPPER}" << EOF
#!/bin/sh
mkdir -p "${LOG_DIR}"
LOG_FILE="${LOG_DIR}/${APP_NAME}-\$(date +%Y%m%d).log"
exec "${APP_BIN}" --config "${CONFIG_FILE}" >> "\${LOG_FILE}" 2>&1
EOF
chmod 755 "${SERVICE_WRAPPER}"

# ==================== 生成系统服务 ====================
echo "-- 配置系统服务..."
cat > "${INIT_SERVICE}" << EOF
#!/bin/sh /etc/rc.common
START=99
STOP=10
USE_PROCD=1

start_service() {
    procd_open_instance
    # 为实现一些日志方面功能，使用包装脚本启动
    procd_set_param command ${SERVICE_WRAPPER}
    # 崩溃自启
    procd_set_param respawn
    procd_close_instance
}

stop_service() {
    killall som-rmqtt-sub 2>/dev/null
}
EOF
chmod 755 "${INIT_SERVICE}"

# ==================== 配置日志清理 ====================
echo "-- 配置日志自动清理..."
(crontab -l 2>/dev/null | grep -v "${APP_NAME}"; echo "0 2 * * * find ${LOG_DIR} -name '${APP_NAME}-*.log' -mtime +7 -delete") | crontab -
/etc/init.d/cron enable
/etc/init.d/cron restart

# ==================== 部署完成 ====================
echo ""
echo "============================================="
echo "部署完成！"
echo "- 主程序路径：${APP_BIN}"
echo "- 配置文件：${CONFIG_FILE}"
echo "- 启动脚本：${SERVICE_WRAPPER}"
echo "- 日志路径：${LOG_DIR}（保留7天，每日自动清理）"
echo "- 服务命令：/etc/init.d/${APP_NAME} start|stop|restart|enable"
echo "- 注意：修改配置后，必须重启服务生效！"
echo "============================================="