import { useState, useEffect, useRef } from "react";
import type { Window } from "@tauri-apps/api/window";
import {
  RefreshCw,
  Minimize2,
  Maximize2,
  X,
  Minus,
  Square,
  Pin,
  PinOff,
  LayoutGrid,
  FolderOpen,
  Github,
  HardDrive,
} from "lucide-react";
import React from "react";
import type { DataSourceInfo } from "../hooks/useUsage";

// OpenTelemetry icon for telemetry data source
const OpenTelemetryIcon = ({ className }: { className?: string }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 128 128"
    className={className}
    fill="currentColor"
  >
    <path
      fill="#f5a800"
      d="M67.648 69.797c-5.246 5.25-5.246 13.758 0 19.008c5.25 5.246 13.758 5.246 19.004 0c5.25-5.25 5.25-13.758 0-19.008c-5.246-5.246-13.754-5.246-19.004 0m14.207 14.219a6.65 6.65 0 0 1-9.41 0a6.65 6.65 0 0 1 0-9.407a6.65 6.65 0 0 1 9.41 0c2.598 2.586 2.598 6.809 0 9.407M86.43 3.672l-8.235 8.234a4.17 4.17 0 0 0 0 5.875l32.149 32.149a4.17 4.17 0 0 0 5.875 0l8.234-8.235c1.61-1.61 1.61-4.261 0-5.87L92.29 3.671a4.16 4.16 0 0 0-5.86 0ZM28.738 108.895a3.763 3.763 0 0 0 0-5.31l-4.183-4.187a3.77 3.77 0 0 0-5.313 0l-8.644 8.649l-.016.012l-2.371-2.375c-1.313-1.313-3.45-1.313-4.75 0c-1.313 1.312-1.313 3.449 0 4.75l14.246 14.242a3.353 3.353 0 0 0 4.746 0c1.3-1.313 1.313-3.45 0-4.746l-2.375-2.375l.016-.012Zm0 0"
    />
    <path
      fill="#425cc7"
      d="M72.297 27.313L54.004 45.605c-1.625 1.625-1.625 4.301 0 5.926L65.3 62.824c7.984-5.746 19.18-5.035 26.363 2.153l9.148-9.149c1.622-1.625 1.622-4.297 0-5.922L78.22 27.313a4.185 4.185 0 0 0-5.922 0ZM60.55 67.585l-6.672-6.672c-1.563-1.562-4.125-1.562-5.684 0l-23.53 23.54a4.036 4.036 0 0 0 0 5.687l13.331 13.332a4.036 4.036 0 0 0 5.688 0l15.132-15.157c-3.199-6.609-2.625-14.593 1.735-20.73m0 0"
    />
  </svg>
);

// Data source icon component
const DataSourceIcon = ({
  sourceType,
  className,
}: {
  sourceType: string;
  className?: string;
}) => {
  if (sourceType === "telemetry") {
    return <OpenTelemetryIcon className={className} />;
  }
  return <HardDrive className={className} />;
};

interface TitleBarProps {
  onRefresh: () => void;
  onToggleCompact: () => void;
  isRefreshing: boolean;
  isCompact?: boolean;
  activeView?: "overall" | "projects";
  onToggleView?: () => void;
  dataSource?: DataSourceInfo;
}

export function TitleBar({
  onRefresh,
  onToggleCompact,
  isRefreshing,
  isCompact = false,
  activeView,
  onToggleView,
  dataSource,
}: TitleBarProps) {
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindowRef = useRef<Window | null>(null);

  useEffect(() => {
    // Dynamically import to avoid issues in non-Tauri environments
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => {
        const appWindow = getCurrentWindow();
        appWindowRef.current = appWindow;

        // Check initial always on top state
        appWindow.isAlwaysOnTop().then(setIsAlwaysOnTop);
        // Check initial maximized state
        appWindow.isMaximized().then(setIsMaximized);

        // Listen for maximize state changes
        const unlisten = appWindow.onResized(() => {
          appWindow.isMaximized().then(setIsMaximized);
        });

        return () => {
          unlisten.then((fn) => fn());
        };
      })
      .catch(() => {
        // Not in Tauri environment, ignore
      });
  }, []);

  const handleMinimize = () => {
    appWindowRef.current?.minimize();
  };

  const handleMaximize = () => {
    appWindowRef.current?.toggleMaximize();
  };

  const handleClose = () => {
    appWindowRef.current?.close();
  };

  const handleToggleAlwaysOnTop = async () => {
    if (!appWindowRef.current) return;
    const newValue = !isAlwaysOnTop;
    await appWindowRef.current.setAlwaysOnTop(newValue);
    setIsAlwaysOnTop(newValue);
  };

  const handleOpenGithub = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open("https://github.com/LyndonWangWork/Claude-Code-Usage-Tracker");
    } catch {
      // Fallback for non-Tauri environment
      window.open(
        "https://github.com/LyndonWangWork/Claude-Code-Usage-Tracker",
        "_blank"
      );
    }
  };

  if (isCompact) {
    return (
      <div
        data-tauri-drag-region
        className="h-6 bg-gray-900 border-b border-gray-700 flex items-center px-2 select-none"
      >
        {/* Logo */}
        <img src="/icon.png" alt="CCT" className="w-4 h-4" draggable={false} />

        {/* Drag Area */}
        <div className="flex-1" data-tauri-drag-region />

        {/* Controls */}
        <div className="flex items-center">
          {/* View Toggle - Only in compact mode */}
          {onToggleView && (
            <button
              onClick={onToggleView}
              className="p-1 hover:bg-gray-700 rounded transition-colors"
              title={
                activeView === "overall"
                  ? "Switch to Projects"
                  : "Switch to Overall"
              }
            >
              {activeView === "overall" ? (
                <FolderOpen className="w-3 h-3 text-gray-400" />
              ) : (
                <LayoutGrid className="w-3 h-3 text-gray-400" />
              )}
            </button>
          )}

          {/* Data Source */}
          {dataSource && (
            <span
              className="p-1 text-gray-400"
              title={
                dataSource.sourceType === "telemetry"
                  ? `${dataSource.displayName} (Port: ${dataSource.collectorPort})`
                  : dataSource.displayName
              }
            >
              <DataSourceIcon
                sourceType={dataSource.sourceType}
                className="w-3 h-3"
              />
            </span>
          )}

          {/* GitHub */}
          <button
            onClick={handleOpenGithub}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
            title="GitHub"
          >
            <Github className="w-3 h-3 text-gray-400" />
          </button>

          {/* Mini Mode */}
          <button
            onClick={onToggleCompact}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
            title="Mini mode"
          >
            <Minimize2 className="w-3 h-3 text-gray-400" />
          </button>

          {/* Always on Top */}
          <button
            onClick={handleToggleAlwaysOnTop}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
            title={isAlwaysOnTop ? "Unpin from top" : "Pin to top"}
          >
            {isAlwaysOnTop ? (
              <Pin className="w-3 h-3 text-blue-400" />
            ) : (
              <PinOff className="w-3 h-3 text-gray-400" />
            )}
          </button>

          {/* Close */}
          <button
            onClick={handleClose}
            className="p-1 hover:bg-red-600 rounded transition-colors"
            title="Close"
          >
            <X className="w-3 h-3 text-gray-400" />
          </button>
        </div>
      </div>
    );
  }

  return (
    <div
      data-tauri-drag-region
      className="h-10 bg-gray-900 border-b border-gray-700 flex items-center px-3 select-none"
    >
      {/* Left: Logo & Name */}
      <div className="flex items-center gap-2" data-tauri-drag-region>
        <img src="/icon.png" alt="CCT" className="w-5 h-5" draggable={false} />
        <span className="text-blue-300 font-semibold text-sm tracking-wide">
          CCT
        </span>
      </div>

      {/* Center: Drag Area */}
      <div className="flex-1" data-tauri-drag-region />

      {/* Right: Actions & Window Controls */}
      <div className="flex items-center gap-1">
        {/* Data Source */}
        {dataSource && (
          <span
            className="p-1.5 text-gray-400"
            title={
              dataSource.sourceType === "telemetry"
                ? `${dataSource.displayName} (Port: ${dataSource.collectorPort})`
                : dataSource.displayName
            }
          >
            <DataSourceIcon
              sourceType={dataSource.sourceType}
              className="w-4 h-4"
            />
          </span>
        )}

        {/* GitHub */}
        <button
          onClick={handleOpenGithub}
          className="p-1.5 hover:bg-gray-700 rounded transition-colors"
          title="GitHub"
        >
          <Github className="w-4 h-4 text-gray-400" />
        </button>

        {/* Refresh */}
        <button
          onClick={onRefresh}
          className="p-1.5 hover:bg-gray-700 rounded transition-colors"
          title="Refresh data"
        >
          <RefreshCw
            className={`w-4 h-4 text-gray-400 ${
              isRefreshing ? "animate-spin" : ""
            }`}
          />
        </button>

        {/* Compact */}
        <button
          onClick={onToggleCompact}
          className="p-1.5 hover:bg-gray-700 rounded transition-colors"
          title="Compact mode"
        >
          <Minimize2 className="w-4 h-4 text-gray-400" />
        </button>

        {/* Always on Top */}
        <button
          onClick={handleToggleAlwaysOnTop}
          className="p-1.5 hover:bg-gray-700 rounded transition-colors"
          title={isAlwaysOnTop ? "Unpin from top" : "Pin to top"}
        >
          {isAlwaysOnTop ? (
            <Pin className="w-4 h-4 text-blue-400" />
          ) : (
            <PinOff className="w-4 h-4 text-gray-400" />
          )}
        </button>

        {/* Separator */}
        <div className="w-px h-4 bg-gray-700 mx-1" />

        {/* Minimize */}
        <button
          onClick={handleMinimize}
          className="p-1.5 hover:bg-gray-700 rounded transition-colors"
          title="Minimize"
        >
          <Minus className="w-4 h-4 text-gray-400" />
        </button>

        {/* Maximize/Restore */}
        <button
          onClick={handleMaximize}
          className="p-1.5 hover:bg-gray-700 rounded transition-colors"
          title={isMaximized ? "Restore" : "Maximize"}
        >
          <Square className="w-4 h-4 text-gray-400" />
        </button>

        {/* Close */}
        <button
          onClick={handleClose}
          className="p-1.5 hover:bg-red-600 rounded transition-colors"
          title="Close"
        >
          <X className="w-4 h-4 text-gray-400" />
        </button>
      </div>
    </div>
  );
}
