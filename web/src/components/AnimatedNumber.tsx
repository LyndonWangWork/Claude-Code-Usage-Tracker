import { useState, useEffect, useRef } from "react";
import React from "react";

interface AnimatedNumberProps {
  /** The target value to animate to */
  value: number;
  /** Optional formatter function to format the displayed value */
  formatter?: (value: number) => string;
  /** Animation duration in milliseconds (default: 500) */
  duration?: number;
  /** Additional CSS classes */
  className?: string;
}

/**
 * A component that animates number changes with a smooth transition.
 * Uses requestAnimationFrame for 60fps smooth animation.
 */
export function AnimatedNumber({
  value,
  formatter,
  duration = 500,
  className,
}: AnimatedNumberProps) {
  const [displayValue, setDisplayValue] = useState(value);
  const prevValueRef = useRef(value);
  const animationRef = useRef<number | null>(null);

  useEffect(() => {
    const startValue = prevValueRef.current;
    const endValue = value;
    const startTime = performance.now();

    // Cancel any ongoing animation
    if (animationRef.current !== null) {
      cancelAnimationFrame(animationRef.current);
    }

    // If value hasn't changed, skip animation
    if (startValue === endValue) {
      return;
    }

    const animate = (currentTime: number) => {
      const elapsed = currentTime - startTime;
      const progress = Math.min(elapsed / duration, 1);

      // easeOutQuad: decelerating to zero velocity
      const eased = 1 - (1 - progress) * (1 - progress);
      const current = startValue + (endValue - startValue) * eased;

      setDisplayValue(current);

      if (progress < 1) {
        animationRef.current = requestAnimationFrame(animate);
      } else {
        // Animation complete, update the ref
        prevValueRef.current = endValue;
        animationRef.current = null;
      }
    };

    animationRef.current = requestAnimationFrame(animate);

    // Cleanup on unmount or when value changes
    return () => {
      if (animationRef.current !== null) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [value, duration]);

  // Update ref when component unmounts with latest value
  useEffect(() => {
    return () => {
      prevValueRef.current = value;
    };
  }, [value]);

  const formatted = formatter
    ? formatter(displayValue)
    : displayValue.toLocaleString();

  return <span className={className}>{formatted}</span>;
}
