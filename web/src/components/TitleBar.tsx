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
} from "lucide-react";
import React from "react";

interface TitleBarProps {
  onRefresh: () => void;
  onToggleCompact: () => void;
  isRefreshing: boolean;
  isCompact?: boolean;
  activeView?: "overall" | "projects";
  onToggleView?: () => void;
}

export function TitleBar({
  onRefresh,
  onToggleCompact,
  isRefreshing,
  isCompact = false,
  activeView,
  onToggleView,
}: TitleBarProps) {
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindowRef = useRef<Window | null>(null);

  useEffect(() => {
    // Dynamically import to avoid issues in non-Tauri environments
    import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
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
    }).catch(() => {
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
      window.open("https://github.com/LyndonWangWork/Claude-Code-Usage-Tracker", "_blank");
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
              title={activeView === "overall" ? "Switch to Projects" : "Switch to Overall"}
            >
              {activeView === "overall" ? (
                <FolderOpen className="w-3 h-3 text-gray-400" />
              ) : (
                <LayoutGrid className="w-3 h-3 text-gray-400" />
              )}
            </button>
          )}

          {/* GitHub */}
          <button
            onClick={handleOpenGithub}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
            title="GitHub"
          >
            <Github className="w-3 h-3 text-gray-400" />
          </button>

          {/* Expand */}
          <button
            onClick={onToggleCompact}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
            title="Expand"
          >
            <Maximize2 className="w-3 h-3 text-gray-400" />
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
            className={`w-4 h-4 text-gray-400 ${isRefreshing ? "animate-spin" : ""}`}
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
