import { Calendar, Hash, Loader2 } from "lucide-react";
import { UsageData, formatTokens, formatCost } from "../hooks/useUsage";
import React from "react";
import { AnimatedNumber } from "./AnimatedNumber";

interface MiniViewProps {
  data: UsageData | null;
  loading: boolean;
  error: string | null;
  onRestore: () => void;
  onMouseEnter: () => void;
}

export function MiniView({ data, loading, error, onRestore, onMouseEnter }: MiniViewProps) {
  // Base container for all states (ensures consistent size and draggability)
  const baseContainerProps = {
    "data-tauri-drag-region": true,
    className: "h-[32px] bg-[#1e1e1e] text-gray-100 flex items-center px-3 select-none cursor-move border border-gray-700 rounded",
    onDoubleClick: onRestore,
    onMouseEnter: onMouseEnter,
  };

  // Loading state
  if (loading) {
    return (
      <div {...baseContainerProps}>
        <Loader2 className="w-4 h-4 text-blue-400 animate-spin" />
        <span className="text-xs text-gray-400 ml-2">Loading...</span>
        <div className="flex-1" data-tauri-drag-region />
      </div>
    );
  }

  // Error state
  if (error || !data) {
    return (
      <div {...baseContainerProps}>
        <span className="text-xs text-red-400">Error loading data</span>
        <div className="flex-1" data-tauri-drag-region />
      </div>
    );
  }

  const todayCost = data.overallStats.todayStats?.costUsd || 0;
  const todayTokens = data.overallStats.todayStats?.totalTokens || 0;

  return (
    <div {...baseContainerProps}>
      {/* Today Cost */}
      <div className="flex items-center gap-1.5" data-tauri-drag-region>
        <Calendar className="w-3.5 h-3.5 text-cyan-400 flex-shrink-0" />
        <span className="text-xs text-gray-400">Today $</span>
        <span className="text-xs text-cyan-300 font-medium">
          <AnimatedNumber value={todayCost} formatter={formatCost} duration={300} />
        </span>
      </div>

      {/* Separator */}
      <div className="w-px h-4 bg-gray-600 mx-3 flex-shrink-0" data-tauri-drag-region />

      {/* Today Tokens */}
      <div className="flex items-center gap-1.5" data-tauri-drag-region>
        <Hash className="w-3.5 h-3.5 text-purple-400 flex-shrink-0" />
        <span className="text-xs text-gray-400">Today Tk</span>
        <span className="text-xs text-purple-300 font-medium">
          <AnimatedNumber value={todayTokens} formatter={formatTokens} duration={300} />
        </span>
      </div>

      {/* Drag area - takes remaining space */}
      <div className="flex-1 min-w-2" data-tauri-drag-region />
    </div>
  );
}
