import { describe, expect, it } from "bun:test";
import type { DayTimetable, ParsedLesson, TimeUnit, WeekTimetable } from "../../utils/untis.ts";
import { buildSplitLaneIndex, getSelectedLessonRange, indexLessonsByPeriod } from "./model.ts";

function lesson(overrides: Partial<ParsedLesson> & { instanceId: string }): ParsedLesson {
  return {
    instanceId: overrides.instanceId,
    subject: overrides.subject ?? "Math",
    subjectLongName: overrides.subjectLongName ?? "Mathematics",
    lessonText: overrides.lessonText ?? "",
    cellState: overrides.cellState ?? "STANDARD",
    teacher: overrides.teacher ?? "T",
    teacherLongName: overrides.teacherLongName ?? "Teacher",
    allTeachers: overrides.allTeachers ?? [overrides.teacher ?? "T"],
    allTeacherLongNames: overrides.allTeacherLongNames ?? [overrides.teacherLongName ?? "Teacher"],
    room: overrides.room ?? "R1",
    roomLongName: overrides.roomLongName ?? "Room 1",
    allClasses: overrides.allClasses ?? ["4AHIF"],
    startTime: overrides.startTime ?? "08:00",
    endTime: overrides.endTime ?? "08:45",
    cancelled: overrides.cancelled ?? false,
    substitution: overrides.substitution ?? false,
    remarks: overrides.remarks ?? "",
  };
}

function getLessonIndex(
  entries: ReturnType<typeof indexLessonsByPeriod>[number],
  startTime: string,
  lessonInstanceId: string,
): number {
  const inPeriod = entries.get(startTime) ?? [];
  return inPeriod.findIndex((entry) => entry.lessonInstanceId === lessonInstanceId);
}

describe("timetable overlap model", () => {
  it("indexes staggered overlaps by actual interval intersection", () => {
    const timegrid: TimeUnit[] = [
      { name: "P1", startTime: "08:00", endTime: "08:30" },
      { name: "P2", startTime: "08:30", endTime: "09:00" },
      { name: "P3", startTime: "09:00", endTime: "09:30" },
    ];

    const day: DayTimetable = {
      date: new Date(2026, 1, 9),
      dayName: "Monday",
      lessons: [
        lesson({ instanceId: "a", subject: "Math", startTime: "08:00", endTime: "09:00" }),
        lesson({ instanceId: "b", subject: "Bio", startTime: "08:30", endTime: "09:30" }),
      ],
    };

    const dayIndex = indexLessonsByPeriod([day], timegrid)[0]!;

    const p1 = dayIndex.get("08:00") ?? [];
    const p2 = dayIndex.get("08:30") ?? [];
    const p3 = dayIndex.get("09:00") ?? [];

    expect(p1.map((e) => e.lessonInstanceId)).toEqual(["a"]);
    expect(p2.map((e) => e.lessonInstanceId)).toEqual(["b", "a"]);
    expect(p3.map((e) => e.lessonInstanceId)).toEqual(["b"]);

    expect(p1[0]?.continuation).toBe("start");
    expect(p2.find((e) => e.lessonInstanceId === "a")?.continuation).toBe("end");
    expect(p2.find((e) => e.lessonInstanceId === "b")?.continuation).toBe("start");
    expect(p3[0]?.continuation).toBe("end");
  });

  it("keeps identical duplicates distinct by instance id", () => {
    const timegrid: TimeUnit[] = [
      { name: "P1", startTime: "08:00", endTime: "08:45" },
    ];

    const day: DayTimetable = {
      date: new Date(2026, 1, 10),
      dayName: "Tuesday",
      lessons: [
        lesson({ instanceId: "dup-a", subject: "Chem", room: "Lab" }),
        lesson({ instanceId: "dup-b", subject: "Chem", room: "Lab" }),
      ],
    };

    const dayIndex = indexLessonsByPeriod([day], timegrid)[0]!;
    const p1 = dayIndex.get("08:00") ?? [];

    expect(p1).toHaveLength(2);
    expect(new Set(p1.map((e) => e.lessonInstanceId))).toEqual(new Set(["dup-a", "dup-b"]));
  });

  it("does not merge back-to-back lessons with same metadata", () => {
    const timegrid: TimeUnit[] = [
      { name: "P1", startTime: "08:00", endTime: "08:45" },
      { name: "P2", startTime: "08:45", endTime: "09:30" },
    ];

    const day: DayTimetable = {
      date: new Date(2026, 1, 11),
      dayName: "Wednesday",
      lessons: [
        lesson({ instanceId: "first", subject: "Eng", room: "R2", startTime: "08:00", endTime: "08:45" }),
        lesson({ instanceId: "second", subject: "Eng", room: "R2", startTime: "08:45", endTime: "09:30" }),
      ],
    };

    const dayIndex = indexLessonsByPeriod([day], timegrid);
    const data: WeekTimetable = { days: [day], timegrid };

    const firstRange = getSelectedLessonRange(data, dayIndex, 0, 0, 0);
    const secondRange = getSelectedLessonRange(data, dayIndex, 0, 1, 0);

    expect(firstRange?.lessonInstanceId).toBe("first");
    expect(firstRange?.startPeriodIdx).toBe(0);
    expect(firstRange?.endPeriodIdx).toBe(0);

    expect(secondRange?.lessonInstanceId).toBe("second");
    expect(secondRange?.startPeriodIdx).toBe(1);
    expect(secondRange?.endPeriodIdx).toBe(1);
  });

  it("tracks selected range for overlapped lesson by instance id", () => {
    const timegrid: TimeUnit[] = [
      { name: "P1", startTime: "08:00", endTime: "08:30" },
      { name: "P2", startTime: "08:30", endTime: "09:00" },
      { name: "P3", startTime: "09:00", endTime: "09:30" },
    ];

    const day: DayTimetable = {
      date: new Date(2026, 1, 12),
      dayName: "Thursday",
      lessons: [
        lesson({ instanceId: "a", subject: "Math", startTime: "08:00", endTime: "09:00" }),
        lesson({ instanceId: "b", subject: "Bio", startTime: "08:30", endTime: "09:30" }),
      ],
    };

    const dayIndex = indexLessonsByPeriod([day], timegrid);
    const data: WeekTimetable = { days: [day], timegrid };

    const lessonIdx = getLessonIndex(dayIndex[0]!, "08:30", "b");
    expect(lessonIdx).toBeGreaterThanOrEqual(0);

    const range = getSelectedLessonRange(data, dayIndex, 0, 1, lessonIdx);
    expect(range?.lessonInstanceId).toBe("b");
    expect(range?.startPeriodIdx).toBe(1);
    expect(range?.endPeriodIdx).toBe(2);
  });

  it("prioritizes lessons that start in the current period", () => {
    const timegrid: TimeUnit[] = [
      { name: "P0", startTime: "07:05", endTime: "07:55" },
      { name: "P1", startTime: "08:00", endTime: "08:50" },
      { name: "P2", startTime: "08:55", endTime: "09:45" },
      { name: "P3", startTime: "10:00", endTime: "10:50" },
      { name: "P4", startTime: "10:55", endTime: "11:45" },
      { name: "P5", startTime: "11:50", endTime: "12:40" },
      { name: "P6", startTime: "12:45", endTime: "13:35" },
    ];

    const long = lesson({
      instanceId: "long",
      subject: "Advisory",
      startTime: "07:05",
      endTime: "13:00",
    });

    const day: DayTimetable = {
      date: new Date(2026, 1, 11),
      dayName: "Wednesday",
      lessons: [
        long,
        lesson({ instanceId: "u1", subject: "S1", startTime: "08:00", endTime: "08:50" }),
        lesson({ instanceId: "u2", subject: "S2", startTime: "08:55", endTime: "09:45" }),
        lesson({ instanceId: "u3", subject: "S3", startTime: "10:00", endTime: "10:50" }),
        lesson({ instanceId: "u4", subject: "S4", startTime: "10:55", endTime: "11:45" }),
        lesson({ instanceId: "u5", subject: "S5", startTime: "11:50", endTime: "12:40" }),
      ],
    };

    const dayIndex = indexLessonsByPeriod([day], timegrid)[0]!;

    expect(dayIndex.get("08:00")?.map((entry) => entry.lessonInstanceId)).toEqual(["u1", "long"]);
    expect(dayIndex.get("08:55")?.map((entry) => entry.lessonInstanceId)).toEqual(["u2", "long"]);
    expect(dayIndex.get("10:00")?.map((entry) => entry.lessonInstanceId)).toEqual(["u3", "long"]);
    expect(dayIndex.get("10:55")?.map((entry) => entry.lessonInstanceId)).toEqual(["u4", "long"]);
    expect(dayIndex.get("11:50")?.map((entry) => entry.lessonInstanceId)).toEqual(["u5", "long"]);
  });

  it("keeps overlap lane assignment stable across consecutive overlap periods", () => {
    const timegrid: TimeUnit[] = [
      { name: "P1", startTime: "08:00", endTime: "08:50" },
      { name: "P2", startTime: "08:55", endTime: "09:45" },
      { name: "P3", startTime: "10:00", endTime: "10:50" },
    ];

    const day: DayTimetable = {
      date: new Date(2026, 1, 2),
      dayName: "Monday",
      lessons: [
        lesson({ instanceId: "wmc-1", subject: "0WMC", teacher: "UNTEG", room: "---", startTime: "08:55", endTime: "09:45" }),
        lesson({ instanceId: "wmc-2", subject: "0WMC", teacher: "UNTEG", room: "---", startTime: "10:00", endTime: "10:50" }),
        lesson({ instanceId: "ggp", subject: "0GGPGW", teacher: "NITSL", room: "E59-1", startTime: "08:55", endTime: "09:45" }),
        lesson({ instanceId: "wmc-main", subject: "1WMC", teacher: "GRULE", room: "E59-1", startTime: "10:00", endTime: "10:50" }),
      ],
    };

    const dayIndex = indexLessonsByPeriod([day], timegrid)[0]!;
    const split = buildSplitLaneIndex(dayIndex, timegrid);

    expect(split.get("08:55")?.right?.lesson.subject).toBe("0WMC");
    expect(split.get("10:00")?.right?.lesson.subject).toBe("0WMC");
  });

  it("keeps long-running overlap partner in same lane", () => {
    const timegrid: TimeUnit[] = [
      { name: "P1", startTime: "08:00", endTime: "08:50" },
      { name: "P2", startTime: "08:55", endTime: "09:45" },
      { name: "P3", startTime: "10:00", endTime: "10:50" },
    ];

    const day: DayTimetable = {
      date: new Date(2026, 1, 5),
      dayName: "Thursday",
      lessons: [
        lesson({ instanceId: "dbi", subject: "0DBI", teacher: "AIST", room: "E59-1", startTime: "08:00", endTime: "08:50" }),
        lesson({ instanceId: "insy", subject: "1INSY", teacher: "TUMF", room: "135", startTime: "08:00", endTime: "10:50" }),
        lesson({ instanceId: "nwp-1", subject: "0NWP", teacher: "DELL", room: "E09", startTime: "08:55", endTime: "09:45" }),
        lesson({ instanceId: "nwp-2", subject: "0NWP", teacher: "DELL", room: "E09", startTime: "10:00", endTime: "10:50" }),
      ],
    };

    const dayIndex = indexLessonsByPeriod([day], timegrid)[0]!;
    const split = buildSplitLaneIndex(dayIndex, timegrid);

    expect(split.get("08:55")?.right?.lesson.subject).toBe("1INSY");
    expect(split.get("10:00")?.right?.lesson.subject).toBe("1INSY");
  });
});
