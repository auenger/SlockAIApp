# Feature: feat-lan-a2a-server LAN A2A 局域网互联

## Basic Information
- **ID**: feat-lan-a2a-server
- **Name**: LAN A2A 局域网互联（方式1 GUI + 方式2 Headless）
- **Priority**: 80
- **Size**: L (split)
- **Dependencies**: feat-a2a-adapter, feat-a2a-remote-client
- **Parent**: null
- **Children**: feat-lan-a2a-access, feat-lan-headless-serve
- **Created**: 2026-04-16

## Description

让局域网内的多台电脑通过 A2A 协议互联，互相访问对方的 Claude Code Agent。

包含两种使用方式：
- **方式1（GUI）**：在 AgentsZone 桌面应用中开启 "LAN Access" 开关，其他设备可连接
- **方式2（Headless）**：通过 `agentszone serve` CLI 命令启动无 GUI 的 A2A 服务

本 feature 为 split parent，具体实现由子 feature 完成。

## User Value Points

### V1: 局域网 Agent 互联
多台电脑的 Claude Code 可以互相协作，一台机器的 Agent 可以调用另一台机器的 Agent。

## Children

### feat-lan-a2a-access（方式1）
TCP 服务循环 + GUI LAN 开关。优先实现。

### feat-lan-headless-serve（方式2）
Headless CLI 模式。依赖 feat-lan-a2a-access 的 TCP 服务代码。

## Context Analysis

### Related Features (deferred)
- feat-lan-a2a-bridge — 独立轻量 a2a-bridge 二进制（方式3）
- feat-lan-mdns-discovery — mDNS 零配置自动发现（方案B）
- feat-lan-security — 安全加固 TLS + 认证（方案C）
