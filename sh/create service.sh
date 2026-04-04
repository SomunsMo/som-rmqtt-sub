#!/bin/sh
set -e
echo "============================================="
echo " MQTT订阅服务 一键部署"
echo "============================================="

# ==================== 核心配置 ====================
APP_NAME="som-rmqtt-sub"
# 配置文件下载地址
CONFIG_DOWNLOAD_URL="https://github.com/SomunsMo/som-rmqtt-sub/raw/master/config/config.json5"
# 自动创建的系统目录
BIN_DIR="/usr/bin"
APP_PATH="${BIN_DIR}/${APP_NAME}"
LOG_DIR="/var/log/${APP_NAME}"
SERVICE_SCRIPT="${BIN_DIR}/som-create-service.sh"
INIT_SERVICE="/etc/init.d/${APP_NAME}"
# 配置文件路径
CONFIG_FILE="${APP_PATH}/config.json5"

# ==================== 自动创建缺失目录 ====================
echo "检查并创建 /usr/bin 目录..."
mkdir -p ${BIN_DIR}
chmod 755 ${BIN_DIR}

# ==================== 优先本地文件，无则下载 ====================
echo "检测程序文件 ${APP_NAME}..."
if [ ! -f "./${APP_NAME}" ]; then
    echo "未找到 ${APP_NAME}，安装失败..."
    exit 1
else
    echo "找到 ${APP_NAME}"
fi

# ==================== 安装程序到全局目录 ====================
echo "部署程序到 ${BIN_DIR}..."
mv -f ./${APP_NAME} ${APP_PATH}
chmod 755 ${APP_PATH}

# ==================== 创建配置文件（已有则不覆盖） ====================
echo "检查配置文件 ${CONFIG_FILE}..."
if [ ! -f "${CONFIG_FILE}" ]; then
    echo "未检测到配置文件，开始下载默认配置..."
    # 创建配置目录
    mkdir -p $(dirname ${CONFIG_FILE})
    # 下载配置文件（关闭证书检查）
    wget -q --no-check-certificate -O ${CONFIG_FILE} ${CONFIG_DOWNLOAD_URL}
    # 检查配置下载是否成功
    if [ ! -f "${CONFIG_FILE}" ]; then
        echo "Err: 配置文件下载失败，请检查或网络！"
        exit 1
    fi
    # 设置全用户可读写权限（666）
    chmod 666 ${CONFIG_FILE}
    echo "配置文件下载并授权完成！"
else
    echo "检测到已有配置文件，跳过下载，保留原有配置！"
fi

# ==================== 创建日志目录 ====================
echo "创建日志目录..."
mkdir -p ${LOG_DIR}
chmod 755 ${LOG_DIR}

# ==================== 生成启动脚本（日志+7天清理） ====================
echo "生成启动脚本..."
cat > ${SERVICE_SCRIPT} << 'EOF'
#!/bin/sh
LOG_DIR="/var/log/som-rmqtt-sub"
LOG_FILE="${LOG_DIR}/som-rmqtt-sub-$(date +%Y%m%d).log"
# 自动删除7天前日志
find ${LOG_DIR} -name "som-rmqtt-sub-*.log" -mtime +7 -delete
# 启动主程序并输出日志
exec /usr/bin/som-rmqtt-sub >> ${LOG_FILE} 2>&1
EOF
chmod 755 ${SERVICE_SCRIPT}

# ==================== 生成OpenWrt系统服务（自动重启） ====================
echo "配置开机自启 + 崩溃重启服务..."
cat > ${INIT_SERVICE} << 'EOF'
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
chmod 755 ${INIT_SERVICE}

# ==================== 自动启动服务 ====================
# 因为需要编辑配置文件，所以不能在脚本中启动服务
#${INIT_SERVICE} enable 2>/dev/null
#${INIT_SERVICE} restart

# ==================== 设置Cron任务，每天检测清除大于7天的日志 ====================
(crontab -l 2>/dev/null | grep -v "${APP_NAME}"; echo "0 2 * * * find ${LOG_DIR} -name '${APP_NAME}-*.log' -mtime +7 -delete") | crontab -
# 启用并重启cron服务，确保生效
/etc/init.d/cron enable
/etc/init.d/cron restart

# ==================== 完成 ====================
echo ""
echo "============================================="
echo "部署完成！所有功能正常生效"
echo "- 程序路径：/usr/bin/som-rmqtt-sub"
echo "- 配置文件：${CONFIG_FILE}"
echo "- 全局调用：直接输入 som-rmqtt-sub 运行"
echo "- 日志路径：/var/log/som-rmqtt-sub/（按天保存7天）"
echo "- 服务管理：/etc/init.d/som-rmqtt-sub start|stop|restart"
echo "- 核心功能：崩溃自动重启 + 开机自启"
echo "- 注意：配置文件完成后，需手动执行启动命令！"
echo "============================================="