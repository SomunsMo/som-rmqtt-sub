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
APP_BIN="${BIN_DIR}/${APP_NAME}"
CONFIG_DIR="/etc/${APP_NAME}"
CONFIG_FILE="${CONFIG_DIR}/${LOCAL_CONFIG}"
LOG_DIR="/var/log/${APP_NAME}"
SERVICE_WRAPPER="${BIN_DIR}/som-create-service.sh"
INIT_SERVICE="/etc/init.d/${APP_NAME}"

# ==================== 创建目录 ====================
echo "1. 初始化系统目录..."
mkdir -p "${BIN_DIR}" "${CONFIG_DIR}" "${LOG_DIR}"
chmod 755 "${BIN_DIR}" "${CONFIG_DIR}" "${LOG_DIR}"

# ==================== 检测主程序 ====================
echo "2. 检测主程序 ${APP_NAME}..."
if [ ! -f "./${APP_NAME}" ]; then
    echo "错误：未找到当前目录的主程序 ${APP_NAME}，安装终止！"
    exit 1
fi
echo "主程序校验通过"

# ==================== 部署主程序 ====================
echo "3. 部署主程序到 ${APP_BIN}..."
mv -f "./${APP_NAME}" "${APP_BIN}"
chmod 755 "${APP_BIN}"
echo "主程序部署完成"

# ==================== 配置文件处理 ====================
echo "4. 处理配置文件..."
if [ -f "./${LOCAL_CONFIG}" ]; then
    echo "检测到本地配置文件，直接部署到 ${CONFIG_FILE}"
    mv -f "./${LOCAL_CONFIG}" "${CONFIG_FILE}"
    chmod 644 "${CONFIG_FILE}"
else
    echo "未找到本地配置，开始下载默认配置..."
    wget -q --no-check-certificate -O "${CONFIG_FILE}" "${REMOTE_CONFIG_URL}"
    if [ ! -f "${CONFIG_FILE}" ]; then
        echo "错误：配置文件下载失败，请检查网络！安装终止！"
        exit 1
    fi
    chmod 666 "${CONFIG_FILE}"
    echo "远程配置下载完成"
fi

# ==================== 生成启动脚本 ====================
echo "5. 生成启动脚本..."
cat > "${SERVICE_WRAPPER}" << EOF
#!/bin/sh
LOG_FILE="${LOG_DIR}/${APP_NAME}-\$(date +%Y%m%d).log"
exec "${APP_BIN}" >> "\${LOG_FILE}" 2>&1
EOF
chmod 755 "${SERVICE_WRAPPER}"

# ==================== 生成系统服务 ====================
echo "6. 配置系统服务..."
cat > "${INIT_SERVICE}" << EOF
#!/bin/sh /etc/rc.common
START=99
STOP=10
USE_PROCD=1

start_service() {
    procd_open_instance
    procd_set_param command /usr/bin/som-create-service.sh
    procd_set_param respawn  # 崩溃自动重启
    procd_close_instance
}

stop_service() {
    killall som-rmqtt-sub 2>/dev/null
}
EOF
chmod 755 "${INIT_SERVICE}"

# ==================== 配置日志清理 ====================
echo "7. 配置日志自动清理..."
(crontab -l 2>/dev/null | grep -v "${APP_NAME}"; echo "0 2 * * * find ${LOG_DIR} -name '${APP_NAME}-*.log' -mtime +7 -delete") | crontab -
/etc/init.d/cron enable
/etc/init.d/cron restart

# ==================== 部署完成 ====================
echo ""
echo "============================================="
echo "部署完成！所有功能正常生效"
echo "- 程序路径：${APP_BIN}"
echo "- 配置文件：${CONFIG_FILE}"
echo "- 日志路径：${LOG_DIR}（保留7天，每日自动清理）"
echo "- 服务命令：/etc/init.d/${APP_NAME} start|stop|restart|enable"
echo "- 核心功能：崩溃自动重启 + 开机自启"
echo "- 注意：修改配置后，必须重启服务生效！"
echo "============================================="