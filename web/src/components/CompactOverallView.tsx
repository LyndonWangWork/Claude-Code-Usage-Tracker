import { DollarSign, Zap, Loader2, TrendingUp, Calendar, Hash } from "lucide-react";
import { UsageData, formatTokens, formatCost } from "../hooks/useUsage";
import React from "react";
import { AnimatedNumber } from "./AnimatedNumber";

interface CompactOverallViewProps {
  data: UsageData | null;
  loading: boolean;
  error: string | null;
  isAnimating?: boolean;
}

export function CompactOverallView({ data, loading, error }: CompactOverallViewProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-4 h-4 text-blue-400 animate-spin" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="p-2 text-[10px] text-red-400 text-center">
        Error loading data
      </div>
    );
  }

  const { overallStats } = data;
  const burnRate = overallStats.burnRate;

  return (
    <div className="p-2 space-y-1">
      {/* Cost - Total */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1">
          <DollarSign className="w-3 h-3 text-green-400" />
          <span className="text-[10px] text-gray-400">Total</span>
        </div>
        <span className="text-[11px] text-green-300 font-medium">
          <AnimatedNumber value={overallStats.totalCostUsd} formatter={formatCost} duration={300} />
        </span>
      </div>

      {/* Today's Cost */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1">
          <Calendar className="w-3 h-3 text-cyan-400" />
          <span className="text-[10px] text-gray-400">Today $</span>
        </div>
        <span className="text-[11px] text-cyan-300 font-medium">
          <AnimatedNumber value={overallStats.todayStats?.costUsd || 0} formatter={formatCost} duration={300} />
        </span>
      </div>

      {/* Today's Tokens */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1">
          <Hash className="w-3 h-3 text-purple-400" />
          <span className="text-[10px] text-gray-400">Today Tk</span>
        </div>
        <span className="text-[11px] text-purple-300 font-medium">
          <AnimatedNumber value={overallStats.todayStats?.totalTokens || 0} formatter={formatTokens} duration={300} />
        </span>
      </div>

      {/* Burn Rate - Tokens/min */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1">
          <Zap className="w-3 h-3 text-yellow-400" />
          <span className="text-[10px] text-gray-400">Burn Rate</span>
        </div>
        <span className="text-[11px] text-yellow-300 font-medium">
          <AnimatedNumber
            value={burnRate?.tokensPerMinute || 0}
            formatter={(v) => formatTokens(Math.round(v))}
            duration={300}
          />
          <span className="text-[8px] text-gray-500">/m</span>
        </span>
      </div>

      {/* Cost Rate - $/hour */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1">
          <TrendingUp className="w-3 h-3 text-orange-400" />
          <span className="text-[10px] text-gray-400">Cost Rate</span>
        </div>
        <span className="text-[11px] text-orange-300 font-medium">
          <AnimatedNumber
            value={burnRate?.costPerHour || 0}
            formatter={formatCost}
            duration={300}
          />
          <span className="text-[8px] text-gray-500">/h</span>
        </span>
      </div>
    </div>
  );
}
