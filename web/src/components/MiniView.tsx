import { Calendar, Hash, Loader2, Maximize2, Pin, PinOff } from "lucide-react";
import { UsageData, formatTokens, formatCost } from "../hooks/useUsage";
import React, { useState, useEffect, useRef } from "react";
import type { Window } from "@tauri-apps/api/window";
import { AnimatedNumber } from "./AnimatedNumber";

interface MiniViewProps {
  data: UsageData | null;
  loading: boolean;
  error: string | null;
  onRestore: () => void;
  onMouseEnter: () => void;
  onCycleMode: () => void;
}

export function MiniView({ data, loading, error, onRestore, onMouseEnter, onCycleMode }: MiniViewProps) {
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(false);
  const appWindowRef = useRef<Window | null>(null);

  useEffect(() => {
    // Dynamically import to avoid issues in non-Tauri environments
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => {
        const appWindow = getCurrentWindow();
        appWindowRef.current = appWindow;
        // Check initial always on top state
        appWindow.isAlwaysOnTop().then(setIsAlwaysOnTop);
      })
      .catch(() => {
        // Not in Tauri environment, ignore
      });
  }, []);

  const handleToggleAlwaysOnTop = async () => {
    if (!appWindowRef.current) return;
    const newValue = !isAlwaysOnTop;
    await appWindowRef.current.setAlwaysOnTop(newValue);
    setIsAlwaysOnTop(newValue);
  };

  // Base container for all states (ensures consistent size and draggability)
  const baseContainerProps = {
    "data-tauri-drag-region": true,
    className: "h-[32px] bg-[#1e1e1e] text-gray-100 flex items-center px-3 select-none cursor-move border border-gray-700 rounded",
    onDoubleClick: onRestore,
    onMouseEnter: onMouseEnter,
  };

  // Always on top button component
  const AlwaysOnTopButton = () => (
    <button
      onClick={handleToggleAlwaysOnTop}
      className="p-1 hover:bg-gray-700 rounded transition-colors flex-shrink-0"
      title={isAlwaysOnTop ? "Unpin from top" : "Pin to top"}
    >
      {isAlwaysOnTop ? (
        <Pin className="w-3.5 h-3.5 text-blue-400" />
      ) : (
        <PinOff className="w-3.5 h-3.5 text-gray-400" />
      )}
    </button>
  );

  // Mode switch button component
  const ModeSwitchButton = () => (
    <button
      onClick={onCycleMode}
      className="p-1 hover:bg-gray-700 rounded transition-colors flex-shrink-0"
      title="Normal mode"
    >
      <Maximize2 className="w-3.5 h-3.5 text-gray-400" />
    </button>
  );

  // Loading state
  if (loading) {
    return (
      <div {...baseContainerProps}>
        <Loader2 className="w-4 h-4 text-blue-400 animate-spin" data-tauri-drag-region />
        <span className="text-xs text-gray-400 ml-2" data-tauri-drag-region>Loading...</span>
        <div className="flex-1" data-tauri-drag-region />
        <AlwaysOnTopButton />
        <ModeSwitchButton />
      </div>
    );
  }

  // Error state
  if (error || !data) {
    return (
      <div {...baseContainerProps}>
        <span className="text-xs text-red-400" data-tauri-drag-region>Error loading data</span>
        <div className="flex-1" data-tauri-drag-region />
        <AlwaysOnTopButton />
        <ModeSwitchButton />
      </div>
    );
  }

  const todayCost = data.overallStats.todayStats?.costUsd || 0;
  const todayTokens = data.overallStats.todayStats?.totalTokens || 0;

  return (
    <div {...baseContainerProps}>
      {/* Today Cost */}
      <div className="flex items-center gap-1.5" data-tauri-drag-region>
        <Calendar className="w-3.5 h-3.5 text-cyan-400 flex-shrink-0" data-tauri-drag-region />
        <span className="text-xs text-gray-400" data-tauri-drag-region>Cost</span>
        <AnimatedNumber value={todayCost} formatter={formatCost} duration={300} className="text-xs text-cyan-300 font-medium" data-tauri-drag-region />
      </div>

      {/* Separator */}
      <div className="w-px h-4 bg-gray-600 mx-3 flex-shrink-0" data-tauri-drag-region />

      {/* Today Tokens */}
      <div className="flex items-center gap-1.5" data-tauri-drag-region>
        <Hash className="w-3.5 h-3.5 text-purple-400 flex-shrink-0" data-tauri-drag-region />
        <span className="text-xs text-gray-400" data-tauri-drag-region>Token</span>
        <AnimatedNumber value={todayTokens} formatter={formatTokens} duration={300} className="text-xs text-purple-300 font-medium" data-tauri-drag-region />
      </div>

      {/* Drag area - takes remaining space */}
      <div className="flex-1 min-w-2" data-tauri-drag-region />

      {/* Always on top button */}
      <AlwaysOnTopButton />

      {/* Mode switch button */}
      <ModeSwitchButton />
    </div>
  );
}
